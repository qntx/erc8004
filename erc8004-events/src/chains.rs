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
        default_rpc: "https://mainnet.base.org",
        is_testnet: false,
    },
    ChainConfig {
        network: Network::EthereumMainnet,
        name: "Ethereum",
        deployment_block: 24_339_871,
        default_rpc: "https://ethereum-rpc.publicnode.com",
        is_testnet: false,
    },
    ChainConfig {
        network: Network::PolygonMainnet,
        name: "Polygon",
        deployment_block: 73_019_847,
        default_rpc: "https://polygon.gateway.tenderly.co",
        is_testnet: false,
    },
    ChainConfig {
        network: Network::ArbitrumMainnet,
        name: "Arbitrum One",
        deployment_block: 327_832_400,
        default_rpc: "https://arbitrum-one-rpc.publicnode.com",
        is_testnet: false,
    },
    ChainConfig {
        network: Network::CeloMainnet,
        name: "Celo",
        deployment_block: 32_479_428,
        default_rpc: "https://forno.celo.org",
        is_testnet: false,
    },
    ChainConfig {
        network: Network::GnosisMainnet,
        name: "Gnosis",
        deployment_block: 39_025_823,
        default_rpc: "https://rpc.gnosischain.com",
        is_testnet: false,
    },
    ChainConfig {
        network: Network::ScrollMainnet,
        name: "Scroll",
        deployment_block: 15_577_120,
        default_rpc: "https://rpc.scroll.io",
        is_testnet: false,
    },
    ChainConfig {
        network: Network::TaikoMainnet,
        name: "Taiko",
        deployment_block: 871_920,
        default_rpc: "https://rpc.mainnet.taiko.xyz",
        is_testnet: false,
    },
    ChainConfig {
        network: Network::BscMainnet,
        name: "BNB Smart Chain",
        deployment_block: 49_143_533,
        default_rpc: "https://bsc.drpc.org",
        is_testnet: false,
    },
    ChainConfig {
        network: Network::MonadMainnet,
        name: "Monad",
        deployment_block: 56_017_606,
        default_rpc: "https://rpc.monad.xyz",
        is_testnet: false,
    },
    ChainConfig {
        network: Network::AbstractMainnet,
        name: "Abstract",
        deployment_block: 41_233_800,
        default_rpc: "https://api.mainnet.abs.xyz",
        is_testnet: false,
    },
    ChainConfig {
        network: Network::AvalancheMainnet,
        name: "Avalanche",
        deployment_block: 77_893_000,
        default_rpc: "https://api.avax.network/ext/bc/C/rpc",
        is_testnet: false,
    },
    ChainConfig {
        network: Network::LineaMainnet,
        name: "Linea",
        deployment_block: 28_949_707,
        default_rpc: "https://rpc.linea.build",
        is_testnet: false,
    },
    ChainConfig {
        network: Network::MantleMainnet,
        name: "Mantle",
        deployment_block: 91_520_634,
        default_rpc: "https://rpc.mantle.xyz",
        is_testnet: false,
    },
    ChainConfig {
        network: Network::MegaEthMainnet,
        name: "MegaETH",
        deployment_block: 7_833_805,
        default_rpc: "https://rpc.megaeth.com",
        is_testnet: false,
    },
    ChainConfig {
        network: Network::OptimismMainnet,
        name: "Optimism",
        deployment_block: 147_956_461,
        default_rpc: "https://mainnet.optimism.io",
        is_testnet: false,
    },
    // Testnets
    ChainConfig {
        network: Network::BaseSepolia,
        name: "Base Sepolia",
        deployment_block: 24_899_933,
        default_rpc: "https://sepolia.base.org",
        is_testnet: true,
    },
    ChainConfig {
        network: Network::EthereumSepolia,
        name: "Ethereum Sepolia",
        deployment_block: 8_067_632,
        default_rpc: "https://ethereum-sepolia-rpc.publicnode.com",
        is_testnet: true,
    },
    ChainConfig {
        network: Network::PolygonAmoy,
        name: "Polygon Amoy",
        deployment_block: 20_965_364,
        default_rpc: "https://rpc-amoy.polygon.technology",
        is_testnet: true,
    },
    ChainConfig {
        network: Network::ArbitrumSepolia,
        name: "Arbitrum Sepolia",
        deployment_block: 159_589_032,
        default_rpc: "https://sepolia-rollup.arbitrum.io/rpc",
        is_testnet: true,
    },
    ChainConfig {
        network: Network::CeloAlfajores,
        name: "Celo Alfajores",
        deployment_block: 31_382_416,
        default_rpc: "https://alfajores-forno.celo-testnet.org",
        is_testnet: true,
    },
    ChainConfig {
        network: Network::ScrollSepolia,
        name: "Scroll Sepolia",
        deployment_block: 14_050_456,
        default_rpc: "https://sepolia-rpc.scroll.io",
        is_testnet: true,
    },
    ChainConfig {
        network: Network::BscTestnet,
        name: "BSC Testnet",
        deployment_block: 51_893_896,
        default_rpc: "https://bsc-testnet-rpc.publicnode.com",
        is_testnet: true,
    },
    ChainConfig {
        network: Network::MonadTestnet,
        name: "Monad Testnet",
        deployment_block: 10_400_000,
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
        deployment_block: 39_855_448,
        default_rpc: "https://sepolia.optimism.io",
        is_testnet: true,
    },
];

/// Look up a [`ChainConfig`] by chain ID.
#[must_use]
pub fn by_chain_id(chain_id: u64) -> Option<&'static ChainConfig> {
    ALL.iter().find(|c| c.chain_id() == chain_id)
}
