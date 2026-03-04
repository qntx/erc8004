package main

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"strconv"
	"time"
)

var client = &http.Client{Timeout: 20 * time.Second}

type rpcReq struct {
	JSONRPC string `json:"jsonrpc"`
	ID      int    `json:"id"`
	Method  string `json:"method"`
	Params  []any  `json:"params"`
}

type rpcResp struct {
	Result json.RawMessage `json:"result"`
	Error  *rpcError       `json:"error"`
}

type rpcError struct {
	Code    int    `json:"code"`
	Message string `json:"message"`
}

func rpcCall(url, method string, params []any) (*rpcResp, time.Duration, error) {
	body, _ := json.Marshal(rpcReq{"2.0", 1, method, params})
	t0 := time.Now()
	resp, err := client.Post(url, "application/json", bytes.NewReader(body))
	elapsed := time.Since(t0)
	if err != nil {
		return nil, elapsed, err
	}
	defer resp.Body.Close()
	if resp.StatusCode != http.StatusOK {
		return nil, elapsed, fmt.Errorf("HTTP %d", resp.StatusCode)
	}
	data, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, elapsed, err
	}
	var r rpcResp
	if err := json.Unmarshal(data, &r); err != nil {
		return nil, elapsed, err
	}
	return &r, elapsed, nil
}

func toHex(n uint64) string { return "0x" + strconv.FormatUint(n, 16) }

func logFilter(from, to uint64) []any {
	return []any{map[string]string{
		"address":   identityAddr,
		"fromBlock": toHex(from),
		"toBlock":   toHex(to),
	}}
}
