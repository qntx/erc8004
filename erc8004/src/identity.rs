//! Identity Registry operations.
//!
//! The Identity Registry is an ERC-721 contract that provides every agent
//! with a portable, censorship-resistant on-chain identifier. Each agent
//! is uniquely identified by its `agentId` (ERC-721 `tokenId`).
//!
//! This module wraps all read and write functions exposed by the
//! `IdentityRegistryUpgradeable` contract.

use alloy::{
    primitives::{Address, Bytes, U256},
    providers::Provider,
};

use crate::{
    contracts::IdentityRegistry,
    error::{Erc8004Error, Result},
};

/// A handle to the Identity Registry contract bound to a specific provider.
///
/// Created via [`Erc8004::identity()`](crate::Erc8004::identity).
#[derive(Debug)]
pub struct Identity<P> {
    address: Address,
    provider: P,
}

impl<P: Provider> Identity<P> {
    /// Create a new `Identity` handle from a provider and contract address.
    pub(crate) const fn new(provider: P, address: Address) -> Self {
        Self { address, provider }
    }

    /// Register a new agent with no URI (URI can be set later via
    /// [`set_agent_uri`](Self::set_agent_uri)).
    ///
    /// Returns the newly minted `agentId` (`U256`).
    ///
    /// # Errors
    ///
    /// Returns an error if the transaction fails.
    pub async fn register(&self) -> Result<U256> {
        let contract = IdentityRegistry::new(self.address, &self.provider);
        let receipt = contract.register_0().send().await?.get_receipt().await?;
        Self::parse_agent_id(&receipt)
    }

    /// Register a new agent with the given `agentURI`.
    ///
    /// Returns the newly minted `agentId` (`U256`).
    ///
    /// # Errors
    ///
    /// Returns an error if the transaction fails.
    pub async fn register_with_uri(&self, agent_uri: &str) -> Result<U256> {
        let contract = IdentityRegistry::new(self.address, &self.provider);
        let receipt = contract
            .register_1(agent_uri.to_owned())
            .send()
            .await?
            .get_receipt()
            .await?;
        Self::parse_agent_id(&receipt)
    }

    /// Register a new agent with a URI and initial metadata entries.
    ///
    /// Returns the newly minted `agentId` (`U256`).
    ///
    /// # Errors
    ///
    /// Returns an error if the transaction fails.
    pub async fn register_with_metadata(
        &self,
        agent_uri: &str,
        metadata: Vec<IdentityRegistry::MetadataEntry>,
    ) -> Result<U256> {
        let contract = IdentityRegistry::new(self.address, &self.provider);
        let receipt = contract
            .register_2(agent_uri.to_owned(), metadata)
            .send()
            .await?
            .get_receipt()
            .await?;
        Self::parse_agent_id(&receipt)
    }

    /// Update the URI for an existing agent.
    ///
    /// Must be called by the agent owner or an approved operator.
    ///
    /// # Errors
    ///
    /// Returns an error if the transaction fails.
    pub async fn set_agent_uri(&self, agent_id: U256, new_uri: &str) -> Result<()> {
        let contract = IdentityRegistry::new(self.address, &self.provider);
        contract
            .setAgentURI(agent_id, new_uri.to_owned())
            .send()
            .await?
            .get_receipt()
            .await?;
        Ok(())
    }

    /// Set the agent wallet with EIP-712 / ERC-1271 signature verification.
    ///
    /// The `signature` must prove control of `new_wallet`.
    ///
    /// # Errors
    ///
    /// Returns an error if the transaction fails.
    pub async fn set_agent_wallet(
        &self,
        agent_id: U256,
        new_wallet: Address,
        deadline: U256,
        signature: Bytes,
    ) -> Result<()> {
        let contract = IdentityRegistry::new(self.address, &self.provider);
        contract
            .setAgentWallet(agent_id, new_wallet, deadline, signature)
            .send()
            .await?
            .get_receipt()
            .await?;
        Ok(())
    }

    /// Clear the agent wallet (resets to zero address).
    ///
    /// Must be called by the agent owner.
    ///
    /// # Errors
    ///
    /// Returns an error if the transaction fails.
    pub async fn unset_agent_wallet(&self, agent_id: U256) -> Result<()> {
        let contract = IdentityRegistry::new(self.address, &self.provider);
        contract
            .unsetAgentWallet(agent_id)
            .send()
            .await?
            .get_receipt()
            .await?;
        Ok(())
    }

    /// Set a metadata key-value pair for an agent.
    ///
    /// Note: the reserved key `"agentWallet"` cannot be set via this
    /// function; use [`set_agent_wallet`](Self::set_agent_wallet) instead.
    ///
    /// # Errors
    ///
    /// Returns an error if the transaction fails.
    pub async fn set_metadata(&self, agent_id: U256, key: &str, value: Bytes) -> Result<()> {
        let contract = IdentityRegistry::new(self.address, &self.provider);
        contract
            .setMetadata(agent_id, key.to_owned(), value)
            .send()
            .await?
            .get_receipt()
            .await?;
        Ok(())
    }

    /// Check whether `spender` is the owner or an approved operator for the agent.
    ///
    /// Reverts with `ERC721NonexistentToken` if the agent does not exist,
    /// which can also serve as an existence check.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails.
    pub async fn is_authorized_or_owner(&self, spender: Address, agent_id: U256) -> Result<bool> {
        let contract = IdentityRegistry::new(self.address, &self.provider);
        Ok(contract
            .isAuthorizedOrOwner(spender, agent_id)
            .call()
            .await?)
    }

    /// Get the `agentURI` (ERC-721 `tokenURI`) for an agent.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails.
    pub async fn token_uri(&self, agent_id: U256) -> Result<String> {
        let contract = IdentityRegistry::new(self.address, &self.provider);
        Ok(contract.tokenURI(agent_id).call().await?)
    }

    /// Get the owner address of an agent (ERC-721 `ownerOf`).
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails.
    pub async fn owner_of(&self, agent_id: U256) -> Result<Address> {
        let contract = IdentityRegistry::new(self.address, &self.provider);
        Ok(contract.ownerOf(agent_id).call().await?)
    }

    /// Get the agent wallet address for an agent.
    ///
    /// Returns [`Address::ZERO`] if the wallet has not been set.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails.
    pub async fn get_agent_wallet(&self, agent_id: U256) -> Result<Address> {
        let contract = IdentityRegistry::new(self.address, &self.provider);
        Ok(contract.getAgentWallet(agent_id).call().await?)
    }

    /// Get a metadata value by key for an agent.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails.
    pub async fn get_metadata(&self, agent_id: U256, key: &str) -> Result<Bytes> {
        let contract = IdentityRegistry::new(self.address, &self.provider);
        Ok(contract
            .getMetadata(agent_id, key.to_owned())
            .call()
            .await?)
    }

    /// Get the contract version string.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails.
    pub async fn get_version(&self) -> Result<String> {
        let contract = IdentityRegistry::new(self.address, &self.provider);
        Ok(contract.getVersion().call().await?)
    }

    /// Get the ERC-721 token balance for an owner address.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails.
    pub async fn balance_of(&self, owner: Address) -> Result<U256> {
        let contract = IdentityRegistry::new(self.address, &self.provider);
        Ok(contract.balanceOf(owner).call().await?)
    }

    /// Get the EIP-712 domain separator fields.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails.
    pub async fn eip712_domain(&self) -> Result<IdentityRegistry::eip712DomainReturn> {
        let contract = IdentityRegistry::new(self.address, &self.provider);
        Ok(contract.eip712Domain().call().await?)
    }

    /// Parse `agentId` from a transaction receipt's `Registered` event.
    fn parse_agent_id(receipt: &alloy::rpc::types::TransactionReceipt) -> Result<U256> {
        receipt
            .inner
            .logs()
            .iter()
            .find_map(|log| {
                log.log_decode::<IdentityRegistry::Registered>()
                    .ok()
                    .map(|e| e.inner.data.agentId)
            })
            .ok_or(Erc8004Error::MissingRegisteredEvent)
    }
}
