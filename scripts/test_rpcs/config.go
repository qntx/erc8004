package main

import (
	"os"
	"path/filepath"
)

type chainMeta struct {
	Name        string
	DeployBlock uint64
}

var chains = map[uint64]chainMeta{
	1:      {"Ethereum", 24_339_871},
	10:     {"Optimism", 147_514_947},
	56:     {"BSC", 79_027_268},
	100:    {"Gnosis", 44_505_010},
	137:    {"Polygon", 82_458_484},
	143:    {"Monad", 52_952_790},
	2741:   {"Abstract", 39_596_871},
	4326:   {"MegaETH", 7_833_805},
	5000:   {"Mantle", 91_333_846},
	8453:   {"Base", 41_663_783},
	42161:  {"Arbitrum", 428_895_443},
	42220:  {"Celo", 58_396_724},
	43114:  {"Avalanche", 77_389_000},
	59144:  {"Linea", 28_662_553},
	167000: {"Taiko", 4_305_747},
	534352: {"Scroll", 29_432_417},
}

const identityAddr = "0x8004A169FB4a3325136EB29fA0ceB6D2e539a432"

type config struct {
	Chains map[string]chainCfg `toml:"chains"`
}

type chainCfg struct {
	RPCs []string `toml:"rpcs"`
}

func findConfig(name string) string {
	dir, err := os.Getwd()
	if err != nil {
		return name
	}
	for {
		p := filepath.Join(dir, name)
		if _, err := os.Stat(p); err == nil {
			return p
		}
		parent := filepath.Dir(dir)
		if parent == dir {
			return name
		}
		dir = parent
	}
}
