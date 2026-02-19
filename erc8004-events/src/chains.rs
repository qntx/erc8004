//! Static chain configuration for all known ERC-8004 deployments.
//!
//! Each entry pairs an [`erc8004::Network`] variant with operational metadata
//! (deployment block, first-event block, default public RPC) that the SDK
//! itself does not track.

use erc8004::Network;

/// Operational metadata for a single ERC-8004 chain deployment.
#[derive(Debug, Clone, Copy)]
pub struct ChainConfig {
    /// The [`erc8004::Network`] variant (provides chain ID and contract addresses).
    pub network: Network,
    /// Block at which the Identity Registry contract was deployed.
    pub deployment_block: u64,
    /// Block of the first `Registered` / `URIUpdated` event on this chain.
    /// When set, the fetcher starts from here instead of `deployment_block`,
    /// skipping potentially millions of empty blocks.
    pub first_event_block: Option<u64>,
    /// Suggested public RPC endpoint.
    pub default_rpc: &'static str,
    /// Whether this is a testnet deployment.
    pub is_testnet: bool,
}

impl ChainConfig {
    /// Returns the effective starting block for a fresh sync.
    #[must_use]
    pub const fn sync_start_block(&self) -> u64 {
        match self.first_event_block {
            Some(b) => b,
            None => self.deployment_block,
        }
    }

    /// Convenience: the EIP-155 chain ID.
    #[must_use]
    pub const fn chain_id(&self) -> u64 {
        self.network.chain_id()
    }
}

/// All known ERC-8004 chain configurations (single source of truth).
pub const ALL: &[ChainConfig] = &[
    // Mainnets
    ChainConfig {
        network: Network::BaseMainnet,
        deployment_block: 41_663_783,
        first_event_block: Some(42_354_482),
        default_rpc: "https://mainnet.base.org",
        is_testnet: false,
    },
    ChainConfig {
        network: Network::EthereumMainnet,
        deployment_block: 24_339_871,
        first_event_block: None,
        default_rpc: "https://ethereum-rpc.publicnode.com",
        is_testnet: false,
    },
    ChainConfig {
        network: Network::PolygonMainnet,
        deployment_block: 73_019_847,
        first_event_block: None,
        default_rpc: "https://polygon-rpc.com",
        is_testnet: false,
    },
    ChainConfig {
        network: Network::ArbitrumMainnet,
        deployment_block: 327_832_400,
        first_event_block: None,
        default_rpc: "https://arb1.arbitrum.io/rpc",
        is_testnet: false,
    },
    ChainConfig {
        network: Network::CeloMainnet,
        deployment_block: 32_479_428,
        first_event_block: None,
        default_rpc: "https://forno.celo.org",
        is_testnet: false,
    },
    ChainConfig {
        network: Network::GnosisMainnet,
        deployment_block: 39_025_823,
        first_event_block: None,
        default_rpc: "https://rpc.gnosischain.com",
        is_testnet: false,
    },
    ChainConfig {
        network: Network::ScrollMainnet,
        deployment_block: 15_577_120,
        first_event_block: None,
        default_rpc: "https://rpc.scroll.io",
        is_testnet: false,
    },
    ChainConfig {
        network: Network::TaikoMainnet,
        deployment_block: 871_920,
        first_event_block: None,
        default_rpc: "https://rpc.mainnet.taiko.xyz",
        is_testnet: false,
    },
    ChainConfig {
        network: Network::BscMainnet,
        deployment_block: 49_143_533,
        first_event_block: None,
        default_rpc: "https://bsc-rpc.publicnode.com",
        is_testnet: false,
    },
    // Testnets
    ChainConfig {
        network: Network::BaseSepolia,
        deployment_block: 24_899_933,
        first_event_block: None,
        default_rpc: "https://sepolia.base.org",
        is_testnet: true,
    },
    ChainConfig {
        network: Network::EthereumSepolia,
        deployment_block: 8_067_632,
        first_event_block: None,
        default_rpc: "https://ethereum-sepolia-rpc.publicnode.com",
        is_testnet: true,
    },
    ChainConfig {
        network: Network::PolygonAmoy,
        deployment_block: 20_965_364,
        first_event_block: None,
        default_rpc: "https://rpc-amoy.polygon.technology",
        is_testnet: true,
    },
    ChainConfig {
        network: Network::ArbitrumSepolia,
        deployment_block: 159_589_032,
        first_event_block: None,
        default_rpc: "https://sepolia-rollup.arbitrum.io/rpc",
        is_testnet: true,
    },
    ChainConfig {
        network: Network::CeloAlfajores,
        deployment_block: 31_382_416,
        first_event_block: None,
        default_rpc: "https://alfajores-forno.celo-testnet.org",
        is_testnet: true,
    },
    ChainConfig {
        network: Network::ScrollSepolia,
        deployment_block: 14_050_456,
        first_event_block: None,
        default_rpc: "https://sepolia-rpc.scroll.io",
        is_testnet: true,
    },
    ChainConfig {
        network: Network::BscTestnet,
        deployment_block: 51_893_896,
        first_event_block: None,
        default_rpc: "https://bsc-testnet-rpc.publicnode.com",
        is_testnet: true,
    },
];

/// Look up a [`ChainConfig`] by chain ID.
#[must_use]
pub fn by_chain_id(chain_id: u64) -> Option<&'static ChainConfig> {
    ALL.iter().find(|c| c.chain_id() == chain_id)
}
