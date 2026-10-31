//! The top-level [`Erc8004`] client for interacting with ERC-8004 registries.
//!
//! # Usage
//!
//! ```rust,no_run
//! use alloy::providers::ProviderBuilder;
//! use erc8004::{Erc8004, Network};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let provider = ProviderBuilder::new()
//!     .connect_http("https://eth.llamarpc.com".parse()?);
//! let client = Erc8004::new(provider).with_network(Network::EthereumMainnet);
//!
//! let version = client.identity()?.get_version().await?;
//! # Ok(())
//! # }
//! ```

use alloy::{primitives::Address, providers::Provider};

use crate::{
    error::{Erc8004Error, Result},
    identity::Identity,
    networks::{Network, NetworkAddresses},
    reputation::Reputation,
    validation::Validation,
};

/// The main client for interacting with ERC-8004 on-chain registries.
///
/// `Erc8004` is generic over the alloy [`Provider`], which means it
/// works seamlessly with any transport (HTTP, `WebSocket`, IPC) and any
/// signer configuration the user has already set up via
/// [`ProviderBuilder`](alloy::providers::ProviderBuilder).
///
/// # Examples
///
/// ```rust,no_run
/// use alloy::providers::ProviderBuilder;
/// use erc8004::{Erc8004, Network};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let provider = ProviderBuilder::new()
///     .connect_http("https://eth.llamarpc.com".parse()?);
///
/// let client = Erc8004::new(provider)
///     .with_network(Network::EthereumMainnet);
///
/// let version = client.identity()?.get_version().await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Erc8004<P> {
    provider: P,
    identity_address: Option<Address>,
    reputation_address: Option<Address>,
    validation_address: Option<Address>,
}

impl<P: Provider> Erc8004<P> {
    /// Create a new `Erc8004` client wrapping the given alloy provider.
    ///
    /// No registry addresses are configured yet. Use [`with_network`](Self::with_network)
    /// to set addresses from a known network, or [`with_identity_address`](Self::with_identity_address)
    /// etc. for custom deployments.
    #[must_use]
    pub const fn new(provider: P) -> Self {
        Self {
            provider,
            identity_address: None,
            reputation_address: None,
            validation_address: None,
        }
    }

    /// Configure all known registry addresses from a pre-defined [`Network`].
    #[must_use]
    pub const fn with_network(mut self, network: Network) -> Self {
        let addrs = network.addresses();
        self.identity_address = Some(addrs.identity);
        self.reputation_address = Some(addrs.reputation);
        self
    }

    /// Configure all registry addresses from a [`NetworkAddresses`] struct.
    #[must_use]
    pub const fn with_addresses(mut self, addrs: NetworkAddresses) -> Self {
        self.identity_address = Some(addrs.identity);
        self.reputation_address = Some(addrs.reputation);
        self
    }

    /// Set a custom Identity Registry address.
    #[must_use]
    pub const fn with_identity_address(mut self, address: Address) -> Self {
        self.identity_address = Some(address);
        self
    }

    /// Set a custom Reputation Registry address.
    #[must_use]
    pub const fn with_reputation_address(mut self, address: Address) -> Self {
        self.reputation_address = Some(address);
        self
    }

    /// Set a custom Validation Registry address.
    #[must_use]
    pub const fn with_validation_address(mut self, address: Address) -> Self {
        self.validation_address = Some(address);
        self
    }

    /// Get a handle to the Identity Registry.
    ///
    /// # Errors
    ///
    /// Returns [`Erc8004Error::RegistryNotConfigured`] if the identity address
    /// has not been set.
    pub fn identity(&self) -> Result<Identity<&P>> {
        let address = self
            .identity_address
            .ok_or(Erc8004Error::RegistryNotConfigured {
                registry: "identity",
            })?;
        Ok(Identity::new(&self.provider, address))
    }

    /// Get a handle to the Reputation Registry.
    ///
    /// # Errors
    ///
    /// Returns [`Erc8004Error::RegistryNotConfigured`] if the reputation address
    /// has not been set.
    pub fn reputation(&self) -> Result<Reputation<&P>> {
        let address = self
            .reputation_address
            .ok_or(Erc8004Error::RegistryNotConfigured {
                registry: "reputation",
            })?;
        Ok(Reputation::new(&self.provider, address))
    }

    /// Get a handle to the Validation Registry.
    ///
    /// # Errors
    ///
    /// Returns [`Erc8004Error::RegistryNotConfigured`] if the validation address
    /// has not been set.
    pub fn validation(&self) -> Result<Validation<&P>> {
        let address = self
            .validation_address
            .ok_or(Erc8004Error::RegistryNotConfigured {
                registry: "validation",
            })?;
        Ok(Validation::new(&self.provider, address))
    }

    /// Get a reference to the underlying alloy provider.
    #[must_use]
    pub const fn provider(&self) -> &P {
        &self.provider
    }

    /// Consume this client and return the underlying alloy provider.
    #[must_use]
    pub fn into_provider(self) -> P {
        self.provider
    }

    /// Get the configured Identity Registry address, if any.
    #[must_use]
    pub const fn identity_address(&self) -> Option<Address> {
        self.identity_address
    }

    /// Get the configured Reputation Registry address, if any.
    #[must_use]
    pub const fn reputation_address(&self) -> Option<Address> {
        self.reputation_address
    }

    /// Get the configured Validation Registry address, if any.
    #[must_use]
    pub const fn validation_address(&self) -> Option<Address> {
        self.validation_address
    }
}

#[cfg(test)]
mod tests {
    use alloy::primitives::address;
    use alloy::providers::ProviderBuilder;

    use super::*;
    use crate::networks::Network;

    fn test_client() -> Erc8004<impl Provider> {
        let provider = ProviderBuilder::new().connect_http("https://localhost:1".parse().unwrap());
        Erc8004::new(provider)
    }

    #[test]
    fn test_new_has_no_addresses() {
        let client = test_client();
        assert!(client.identity_address().is_none());
        assert!(client.reputation_address().is_none());
        assert!(client.validation_address().is_none());
    }

    #[test]
    fn test_with_network_configures_handles() {
        let client = test_client().with_network(Network::EthereumMainnet);
        let addrs = Network::EthereumMainnet.addresses();
        assert_eq!(client.identity_address(), Some(addrs.identity));
        assert_eq!(client.reputation_address(), Some(addrs.reputation));
        assert!(client.identity().is_ok());
        assert!(client.reputation().is_ok());
    }

    #[test]
    fn test_with_individual_addresses_stored_independently() {
        let id = address!("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
        let rep = address!("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb");
        let val = address!("cccccccccccccccccccccccccccccccccccccccc");
        let client = test_client()
            .with_identity_address(id)
            .with_reputation_address(rep)
            .with_validation_address(val);
        assert_eq!(client.identity_address(), Some(id));
        assert_eq!(client.reputation_address(), Some(rep));
        assert_eq!(client.validation_address(), Some(val));
    }

    #[test]
    fn test_unconfigured_registries_return_error() {
        let client = test_client();
        for (label, result) in [
            ("identity", client.identity().err()),
            ("reputation", client.reputation().err()),
            ("validation", client.validation().err()),
        ] {
            let msg = result.map_or_else(|| unreachable!("{label} should fail"), |e| e.to_string());
            assert!(
                msg.contains("not configured"),
                "{label}: expected 'not configured', got: {msg}"
            );
        }
    }

    #[test]
    fn test_with_addresses_struct() {
        let addrs = NetworkAddresses {
            identity: address!("1111111111111111111111111111111111111111"),
            reputation: address!("2222222222222222222222222222222222222222"),
        };
        let client = test_client().with_addresses(addrs);
        assert_eq!(client.identity_address(), Some(addrs.identity));
        assert_eq!(client.reputation_address(), Some(addrs.reputation));
    }
}
