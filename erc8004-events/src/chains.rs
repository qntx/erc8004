//! Static chain configuration for all known ERC-8004 deployments.
//!
//! Each entry pairs an [`erc8004::Network`] variant with operational metadata
//! (deployment block, default public RPC) that the SDK itself does not track.

use erc8004::Network;

/// Operational metadata for a single ERC-8004 chain deployment.
#[derive(Debug, Clone, Copy)]
pub struct ChainConfig {
    /// The [`erc8004::Network`] variant (provides chain ID and contract addresses).
    pub network: Network,
    /// Human-readable chain name for display purposes.
    pub name: &'static str,
    /// Block at which the Identity Registry contract was deployed.
    pub deployment_block: u64,
    /// Suggested public RPC endpoint.
    pub default_rpc: &'static str,
    /// Whether this is a testnet deployment.
    pub is_testnet: bool,
}

impl ChainConfig {
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
        name: "Base",
        deployment_block: 41_663_783,
        default_rpc: "https://base.gateway.tenderly.co",
        is_testnet: false,
    },
    ChainConfig {
        network: Network::EthereumMainnet,
        name: "Ethereum",
        deployment_block: 24_339_871,
        default_rpc: "https://mainnet.gateway.tenderly.co",
        is_testnet: false,
    },
    ChainConfig {
        network: Network::PolygonMainnet,
        name: "Polygon",
        deployment_block: 82_458_484,
        default_rpc: "https://rpc.sentio.xyz/matic",
        is_testnet: false,
    },
    ChainConfig {
        network: Network::ArbitrumMainnet,
        name: "Arbitrum One",
        deployment_block: 428_895_443,
        default_rpc: "https://rpc.sentio.xyz/arbitrum-one",
        is_testnet: false,
    },
    ChainConfig {
        network: Network::CeloMainnet,
        name: "Celo",
        deployment_block: 58_396_724,
        default_rpc: "https://celo-json-rpc.stakely.io",
        is_testnet: false,
    },
    ChainConfig {
        network: Network::GnosisMainnet,
        name: "Gnosis",
        deployment_block: 44_505_010,
        default_rpc: "https://gnosis-rpc.publicnode.com",
        is_testnet: false,
    },
    ChainConfig {
        network: Network::ScrollMainnet,
        name: "Scroll",
        deployment_block: 29_432_417,
        default_rpc: "https://scroll-rpc.publicnode.com",
        is_testnet: false,
    },
    ChainConfig {
        network: Network::TaikoMainnet,
        name: "Taiko",
        deployment_block: 4_305_747,
        default_rpc: "https://rpc.taiko.xyz",
        is_testnet: false,
    },
    ChainConfig {
        network: Network::BscMainnet,
        name: "BNB Smart Chain",
        deployment_block: 79_027_268,
        default_rpc: "https://public-bsc.nownodes.io",
        is_testnet: false,
    },
    ChainConfig {
        network: Network::MonadMainnet,
        name: "Monad",
        deployment_block: 52_952_790,
        default_rpc: "https://rpc.sentio.xyz/monad-mainnet",
        is_testnet: false,
    },
    ChainConfig {
        network: Network::AbstractMainnet,
        name: "Abstract",
        deployment_block: 39_596_871,
        default_rpc: "https://api.mainnet.abs.xyz",
        is_testnet: false,
    },
    ChainConfig {
        network: Network::AvalancheMainnet,
        name: "Avalanche",
        deployment_block: 77_389_000,
        default_rpc: "https://rpc.sentio.xyz/avalanche",
        is_testnet: false,
    },
    ChainConfig {
        network: Network::LineaMainnet,
        name: "Linea",
        deployment_block: 28_662_553,
        default_rpc: "https://linea-rpc.publicnode.com",
        is_testnet: false,
    },
    ChainConfig {
        network: Network::MantleMainnet,
        name: "Mantle",
        deployment_block: 91_333_846,
        default_rpc: "https://rpc.mantle.xyz",
        is_testnet: false,
    },
    ChainConfig {
        network: Network::MegaEthMainnet,
        name: "MegaETH",
        deployment_block: 7_833_805,
        default_rpc: "https://mainnet.megaeth.com/rpc",
        is_testnet: false,
    },
    ChainConfig {
        network: Network::OptimismMainnet,
        name: "Optimism",
        deployment_block: 147_514_947,
        default_rpc: "https://rpc.sentio.xyz/optimism",
        is_testnet: false,
    },
    // Testnets
    ChainConfig {
        network: Network::BaseSepolia,
        name: "Base Sepolia",
        deployment_block: 36_304_165,
        default_rpc: "https://sepolia.base.org",
        is_testnet: true,
    },
    ChainConfig {
        network: Network::EthereumSepolia,
        name: "Ethereum Sepolia",
        deployment_block: 9_989_393,
        default_rpc: "https://ethereum-sepolia-rpc.publicnode.com",
        is_testnet: true,
    },
    ChainConfig {
        network: Network::PolygonAmoy,
        name: "Polygon Amoy",
        deployment_block: 33_069_064,
        default_rpc: "https://rpc-amoy.polygon.technology",
        is_testnet: true,
    },
    ChainConfig {
        network: Network::ArbitrumSepolia,
        name: "Arbitrum Sepolia",
        deployment_block: 239_945_838,
        default_rpc: "https://sepolia-rollup.arbitrum.io/rpc",
        is_testnet: true,
    },
    ChainConfig {
        network: Network::CeloAlfajores,
        name: "Celo Alfajores",
        deployment_block: 17_013_547,
        default_rpc: "https://alfajores-forno.celo-testnet.org",
        is_testnet: true,
    },
    ChainConfig {
        network: Network::ScrollSepolia,
        name: "Scroll Sepolia",
        deployment_block: 16_543_185,
        default_rpc: "https://sepolia-rpc.scroll.io",
        is_testnet: true,
    },
    ChainConfig {
        network: Network::BscTestnet,
        name: "BSC Testnet",
        deployment_block: 84_555_147,
        default_rpc: "https://bsc-testnet-rpc.publicnode.com",
        is_testnet: true,
    },
    ChainConfig {
        network: Network::MonadTestnet,
        name: "Monad Testnet",
        deployment_block: 10_391_697,
        default_rpc: "https://testnet-rpc.monad.xyz",
        is_testnet: true,
    },
    ChainConfig {
        network: Network::LineaSepolia,
        name: "Linea Sepolia",
        deployment_block: 24_323_547,
        default_rpc: "https://rpc.sepolia.linea.build",
        is_testnet: true,
    },
    ChainConfig {
        network: Network::MantleSepolia,
        name: "Mantle Sepolia",
        deployment_block: 34_586_937,
        default_rpc: "https://rpc.sepolia.mantle.xyz",
        is_testnet: true,
    },
    ChainConfig {
        network: Network::MegaEthTestnet,
        name: "MegaETH Testnet",
        deployment_block: 11_668_749,
        default_rpc: "https://carrot.megaeth.com/rpc",
        is_testnet: true,
    },
    ChainConfig {
        network: Network::OptimismSepolia,
        name: "Optimism Sepolia",
        deployment_block: 34_412_868,
        default_rpc: "https://sepolia.optimism.io",
        is_testnet: true,
    },
];

/// Look up a [`ChainConfig`] by chain ID.
#[must_use]
pub fn by_chain_id(chain_id: u64) -> Option<&'static ChainConfig> {
    ALL.iter().find(|c| c.chain_id() == chain_id)
}
