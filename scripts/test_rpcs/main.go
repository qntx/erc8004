package main

import (
	"flag"
	"fmt"
	"log"
	"maps"
	"os"
	"slices"
	"strconv"
	"strings"
	"sync"

	"github.com/BurntSushi/toml"
)

func main() {
	chainsFlag := flag.String("chains", "", "comma-separated chain IDs to test (default: all)")
	writeFlag := flag.Bool("write", false, "overwrite config.toml with ranked results")
	flag.Parse()

	cfgPath := findConfig("config.toml")

	var cfg config
	if _, err := toml.DecodeFile(cfgPath, &cfg); err != nil {
		log.Fatalf("reading %s: %v", cfgPath, err)
	}

	filter := map[uint64]bool{}
	if *chainsFlag != "" {
		for _, s := range strings.Split(*chainsFlag, ",") {
			id, _ := strconv.ParseUint(strings.TrimSpace(s), 10, 64)
			if id > 0 {
				filter[id] = true
			}
		}
	}

	total := 0
	for _, c := range cfg.Chains {
		total += len(c.RPCs)
	}
	fmt.Printf("ERC-8004 RPC Health Check — %d endpoints across %d chains\n", total, len(cfg.Chains))

	allResults := make(map[uint64][]result)
	var mu sync.Mutex
	var wg sync.WaitGroup

	for cidStr, cc := range cfg.Chains {
		cid, _ := strconv.ParseUint(cidStr, 10, 64)
		if len(filter) > 0 && !filter[cid] {
			continue
		}
		meta, ok := chains[cid]
		if !ok {
			fmt.Printf("  [%6d] unknown chain, skipping\n", cid)
			continue
		}

		wg.Go(func() {
			rpcs := cc.RPCs
			fmt.Printf("  [%6d] %s (%d RPCs) ...\n", cid, meta.Name, len(rpcs))

			results := make([]result, len(rpcs))
			var inner sync.WaitGroup
			for i, u := range rpcs {
				inner.Add(1)
				go func() {
					defer inner.Done()
					results[i] = testEndpoint(u, meta.DeployBlock)
				}()
			}
			inner.Wait()

			n := 0
			for _, r := range results {
				if r.Archive {
					n++
				}
			}
			fmt.Printf("  [%6d] %s done: %d/%d archive-capable\n", cid, meta.Name, n, len(rpcs))

			mu.Lock()
			allResults[cid] = results
			mu.Unlock()
		})
	}
	wg.Wait()

	for _, cid := range slices.Sorted(maps.Keys(allResults)) {
		printChain(cid, chains[cid], allResults[cid])
	}

	tomlOut := generateTOML(allResults)
	fmt.Printf("\n%s\n  RECOMMENDED config.toml\n%s\n\n%s",
		strings.Repeat("─", 90), strings.Repeat("─", 90), tomlOut)

	if *writeFlag {
		if err := os.WriteFile(cfgPath, []byte(tomlOut), 0644); err != nil {
			log.Fatalf("writing %s: %v", cfgPath, err)
		}
		fmt.Printf("  ✅ Written to %s\n", cfgPath)
	} else {
		fmt.Printf("  💡 Pass -write to overwrite %s automatically.\n", cfgPath)
	}
}
