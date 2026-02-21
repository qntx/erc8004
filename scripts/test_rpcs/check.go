package main

import "encoding/json"

type result struct {
	URL       string
	Reachable bool
	LatencyMs float64
	Archive   bool
	Logs      int
	MaxRange  int
	Error     string
}

func checkPing(url string) (ok bool, ms float64, errMsg string) {
	r, d, err := rpcCall(url, "eth_blockNumber", []any{})
	if err != nil {
		return false, 0, truncate(err.Error(), 60)
	}
	if r.Error != nil {
		return false, float64(d.Milliseconds()), truncate(r.Error.Message, 60)
	}
	return true, float64(d.Milliseconds()), ""
}

func checkArchive(url string, deploy uint64) (ok bool, nLogs int, errMsg string) {
	r, _, err := rpcCall(url, "eth_getLogs", logFilter(deploy, deploy+100))
	if err != nil {
		return false, 0, truncate(err.Error(), 60)
	}
	if r.Error != nil {
		return false, 0, truncate(r.Error.Message, 60)
	}
	var logs []json.RawMessage
	if err := json.Unmarshal(r.Result, &logs); err != nil {
		return false, 0, "invalid result"
	}
	if len(logs) == 0 {
		return false, 0, "0 logs at deploy block (silent drop)"
	}
	return true, len(logs), ""
}

var rangeSteps = []int{500, 2_000, 5_000, 10_000, 50_000}

func checkMaxRange(url string, deploy uint64) int {
	best := 0
	for _, r := range rangeSteps {
		resp, _, err := rpcCall(url, "eth_getLogs", logFilter(deploy, deploy+uint64(r)))
		if err != nil || resp.Error != nil {
			break
		}
		best = r
	}
	return best
}

func testEndpoint(url string, deploy uint64) result {
	ok, ms, err := checkPing(url)
	if !ok {
		return result{URL: url, Error: err}
	}
	arc, n, err := checkArchive(url, deploy)
	if !arc {
		return result{URL: url, Reachable: true, LatencyMs: ms, Error: err}
	}
	mx := checkMaxRange(url, deploy)
	return result{URL: url, Reachable: true, LatencyMs: ms, Archive: true, Logs: n, MaxRange: mx}
}

func truncate(s string, n int) string {
	if len(s) > n {
		return s[:n]
	}
	return s
}
