//! Pre-configured network definitions with known contract addresses.
//!
//! ERC-8004 contracts are deployed via CREATE2 deterministic deployment,
//! so mainnet chains share the same addresses and testnet chains share
//! the same addresses.

use alloy::primitives::{Address, address};

/// Known contract addresses for a specific network deployment.
#[derive(Debug, Clone, Copy)]
pub struct NetworkAddresses {
    /// The Identity Registry (ERC-721) contract address.
    pub identity: Address,
    /// The Reputation Registry contract address.
    pub reputation: Address,
}

/// Pre-defined network configurations for ERC-8004 deployments.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Network {
    /// Ethereum Mainnet (chain ID 1).
    EthereumMainnet,
    /// Ethereum Sepolia testnet (chain ID 11155111).
    EthereumSepolia,
    /// Base Mainnet (chain ID 8453).
    BaseMainnet,
    /// Base Sepolia testnet (chain ID 84532).
    BaseSepolia,
    /// Polygon Mainnet (chain ID 137).
    PolygonMainnet,
    /// Polygon Amoy testnet (chain ID 80002).
    PolygonAmoy,
    /// Arbitrum One Mainnet (chain ID 42161).
    ArbitrumMainnet,
    /// Arbitrum Sepolia testnet (chain ID 421614).
    ArbitrumSepolia,
    /// Celo Mainnet (chain ID 42220).
    CeloMainnet,
    /// Celo Alfajores testnet (chain ID 44787).
    CeloAlfajores,
    /// Gnosis Mainnet (chain ID 100).
    GnosisMainnet,
    /// Scroll Mainnet (chain ID 534352).
    ScrollMainnet,
    /// Scroll Sepolia testnet (chain ID 534351).
    ScrollSepolia,
    /// Taiko Mainnet — Alethia (chain ID 167000).
    TaikoMainnet,
    /// Monad Mainnet (chain ID 143).
    MonadMainnet,
    /// Monad Testnet (chain ID 10143).
    MonadTestnet,
    /// BNB Smart Chain Mainnet (chain ID 56).
    BscMainnet,
    /// BNB Smart Chain Testnet (chain ID 97).
    BscTestnet,
}

/// Shared addresses for all mainnet deployments (CREATE2 deterministic).
const MAINNET_IDENTITY: Address = address!("8004A169FB4a3325136EB29fA0ceB6D2e539a432");
const MAINNET_REPUTATION: Address = address!("8004BAa17C55a88189AE136b182e5fdA19dE9b63");

/// Shared addresses for all testnet deployments (CREATE2 deterministic).
const TESTNET_IDENTITY: Address = address!("8004A818BFB912233c491871b3d84c89A494BD9e");
const TESTNET_REPUTATION: Address = address!("8004B663056A597Dffe9eCcC1965A193B7388713");

impl Network {
    /// Returns the known contract addresses for this network.
    #[must_use]
    pub const fn addresses(self) -> NetworkAddresses {
        match self {
            Self::EthereumMainnet
            | Self::BaseMainnet
            | Self::PolygonMainnet
            | Self::ArbitrumMainnet
            | Self::CeloMainnet
            | Self::GnosisMainnet
            | Self::ScrollMainnet
            | Self::TaikoMainnet
            | Self::MonadMainnet
            | Self::BscMainnet => NetworkAddresses {
                identity: MAINNET_IDENTITY,
                reputation: MAINNET_REPUTATION,
            },
            Self::EthereumSepolia
            | Self::BaseSepolia
            | Self::PolygonAmoy
            | Self::ArbitrumSepolia
            | Self::CeloAlfajores
            | Self::ScrollSepolia
            | Self::MonadTestnet
            | Self::BscTestnet => NetworkAddresses {
                identity: TESTNET_IDENTITY,
                reputation: TESTNET_REPUTATION,
            },
        }
    }

    /// Returns the EIP-155 chain ID for this network.
    #[must_use]
    pub const fn chain_id(self) -> u64 {
        match self {
            Self::EthereumMainnet => 1,
            Self::EthereumSepolia => 11_155_111,
            Self::BaseMainnet => 8453,
            Self::BaseSepolia => 84532,
            Self::PolygonMainnet => 137,
            Self::PolygonAmoy => 80002,
            Self::ArbitrumMainnet => 42161,
            Self::ArbitrumSepolia => 421_614,
            Self::CeloMainnet => 42220,
            Self::CeloAlfajores => 44787,
            Self::GnosisMainnet => 100,
            Self::ScrollMainnet => 534_352,
            Self::ScrollSepolia => 534_351,
            Self::TaikoMainnet => 167_000,
            Self::MonadMainnet => 143,
            Self::MonadTestnet => 10143,
            Self::BscMainnet => 56,
            Self::BscTestnet => 97,
        }
    }

    /// All known ERC-8004 network variants.
    pub const ALL: &[Self] = &[
        Self::EthereumMainnet,
        Self::EthereumSepolia,
        Self::BaseMainnet,
        Self::BaseSepolia,
        Self::PolygonMainnet,
        Self::PolygonAmoy,
        Self::ArbitrumMainnet,
        Self::ArbitrumSepolia,
        Self::CeloMainnet,
        Self::CeloAlfajores,
        Self::GnosisMainnet,
        Self::ScrollMainnet,
        Self::ScrollSepolia,
        Self::TaikoMainnet,
        Self::MonadMainnet,
        Self::MonadTestnet,
        Self::BscMainnet,
        Self::BscTestnet,
    ];

    /// Look up a [`Network`] by its EIP-155 chain ID.
    ///
    /// Returns [`None`] if the chain ID is not a known ERC-8004 deployment.
    #[must_use]
    pub fn from_chain_id(chain_id: u64) -> Option<Self> {
        Self::ALL.iter().find(|n| n.chain_id() == chain_id).copied()
    }

    /// Returns the `eip155:{chainId}` namespace prefix for agent registry identifiers.
    #[must_use]
    pub fn agent_registry_prefix(self) -> String {
        format!("eip155:{}:{}", self.chain_id(), self.addresses().identity)
    }
}
