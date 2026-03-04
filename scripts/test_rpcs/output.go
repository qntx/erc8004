package main

import (
	"cmp"
	"fmt"
	"maps"
	"slices"
	"strconv"
	"strings"
	"time"
)

func (r result) icon() string {
	switch {
	case !r.Reachable:
		return "✗"
	case !r.Archive:
		return "△"
	default:
		return "✓"
	}
}

func printChain(cid uint64, meta chainMeta, results []result) {
	sortResults(results)
	fmt.Printf("\n%s\n  %s (chain %d) — %d endpoints\n%s\n",
		strings.Repeat("─", 90), meta.Name, cid, len(results), strings.Repeat("─", 90))
	fmt.Printf(" %2s  %s  %6s  %7s  %9s  %s\n", "#", " ", "Ping", "Archive", "MaxRange", "URL")

	for i, r := range results {
		lat := "  —"
		if r.LatencyMs > 0 {
			lat = fmt.Sprintf("%4.0fms", r.LatencyMs)
		}
		arc := " NO"
		if r.Archive {
			arc = "YES"
		}
		rng := "    —"
		if r.MaxRange > 0 {
			rng = fmt.Sprintf("%7s", fmtInt(r.MaxRange))
		}
		short := strings.TrimPrefix(r.URL, "https://")
		fmt.Printf(" %2d  %s  %6s  %7s  %9s  %s\n", i+1, r.icon(), lat, arc, rng, short)
	}
}

func generateTOML(allResults map[uint64][]result) string {
	var b strings.Builder
	b.WriteString("# ERC-8004 events sync configuration.\n")
	b.WriteString("# RPC endpoints per chain, ordered by priority (best first).\n")
	b.WriteString("# The sync engine tries each in order; on failure it falls back.\n")
	fmt.Fprintf(&b, "# Ranked by test_rpcs at %s\n\n", time.Now().UTC().Format("2006-01-02 15:04 UTC"))

	for _, cid := range slices.Sorted(maps.Keys(allResults)) {
		results := allResults[cid]
		sortResults(results)
		meta := chains[cid]
		fmt.Fprintf(&b, "[chains.%d]  # %s\nrpcs = [\n", cid, meta.Name)
		for _, r := range results {
			if r.Reachable {
				fmt.Fprintf(&b, "    %q,\n", r.URL)
			}
		}
		b.WriteString("]\n\n")
	}
	return b.String()
}

func sortResults(rs []result) {
	slices.SortFunc(rs, func(a, b result) int {
		return cmp.Or(
			cmp.Compare(btoi(a.Archive), btoi(b.Archive)),
			cmp.Compare(b.MaxRange, a.MaxRange),
			cmp.Compare(a.LatencyMs, b.LatencyMs),
		)
	})
}

func btoi(b bool) int {
	if b {
		return 0
	}
	return 1
}

func fmtInt(n int) string {
	s := strconv.Itoa(n)
	if len(s) <= 3 {
		return s
	}
	var b strings.Builder
	pre := len(s) % 3
	if pre > 0 {
		b.WriteString(s[:pre])
	}
	for i := pre; i < len(s); i += 3 {
		if b.Len() > 0 {
			b.WriteByte(',')
		}
		b.WriteString(s[i : i+3])
	}
	return b.String()
}
