//! Identity-registry actions — reads, writes, and receipt helpers.

use alloy::primitives::{Address, Bytes, U256};
use alloy::providers::Provider;
use alloy::rpc::types::TransactionReceipt;

use crate::abi::IIdentityRegistry;
use crate::events::Event;
use crate::wallet::{AgentWalletDeadlineError, check_agent_wallet_deadline};

use super::{Erc8004, ProviderError, TxOutcome};

/// One agent: owner, `agentURI`, and the optional agent wallet (zero address → `None`).
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Agent {
    /// The agent id.
    pub agent_id: U256,
    /// The owner's address.
    pub owner: Address,
    /// The `agentURI` string.
    pub agent_uri: String,
    /// The agent wallet (`None` when unset).
    pub agent_wallet: Option<Address>,
}

/// One `register(string, metadata)` metadata entry.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct MetadataEntry {
    /// Metadata key (`agentWallet` is reserved).
    pub key: String,
    /// Metadata value.
    pub value: Bytes,
}

impl MetadataEntry {
    /// `MetadataEntry { key, value }`.
    #[must_use]
    pub fn new(key: impl Into<String>, value: Bytes) -> Self {
        Self {
            key: key.into(),
            value,
        }
    }
}

/// Identity-registry handle borrowed from [`Erc8004`].
#[derive(Debug)]
pub struct Identity<'a, P> {
    client: &'a Erc8004<P>,
}

impl<'a, P: Provider> Identity<'a, P> {
    pub(crate) const fn new(client: &'a Erc8004<P>) -> Self {
        Self { client }
    }

    const fn registry(&self) -> Address {
        self.client.deployment().identity_registry
    }

    async fn call<C: alloy::sol_types::SolCall>(
        &self,
        call: C,
    ) -> Result<C::Return, ProviderError> {
        self.client.call(self.registry(), call).await
    }

    async fn send<C: alloy::sol_types::SolCall>(
        &self,
        call: C,
    ) -> Result<TransactionReceipt, ProviderError> {
        self.client.send(self.registry(), call).await
    }

    /// `ownerOf` + `tokenURI` + `getAgentWallet` (zero → `None`).
    ///
    /// # Errors
    ///
    /// [`ProviderError::Revert`] on contract reverts (e.g. [`crate::revert::RevertReason::AgentNotFound`]),
    /// [`ProviderError::Rpc`] on transport failures.
    pub async fn get_agent(&self, agent_id: U256) -> Result<Agent, ProviderError> {
        let (owner, agent_uri, wallet) = futures_util::try_join!(
            self.call(IIdentityRegistry::ownerOfCall { agentId: agent_id }),
            self.call(IIdentityRegistry::tokenURICall { agentId: agent_id }),
            self.call(IIdentityRegistry::getAgentWalletCall { agentId: agent_id }),
        )?;
        Ok(Agent {
            agent_id,
            owner,
            agent_uri,
            agent_wallet: (wallet != Address::ZERO).then_some(wallet),
        })
    }

    /// `tokenURI`.
    ///
    /// # Errors
    ///
    /// [`ProviderError::Revert`] on contract reverts, [`ProviderError::Rpc`] on transport failures.
    pub async fn get_agent_uri(&self, agent_id: U256) -> Result<String, ProviderError> {
        self.call(IIdentityRegistry::tokenURICall { agentId: agent_id })
            .await
    }

    /// `ownerOf`.
    ///
    /// # Errors
    ///
    /// [`ProviderError::Revert`] on contract reverts, [`ProviderError::Rpc`] on transport failures.
    pub async fn get_agent_owner(&self, agent_id: U256) -> Result<Address, ProviderError> {
        self.call(IIdentityRegistry::ownerOfCall { agentId: agent_id })
            .await
    }

    /// `getAgentWallet` — `None` when unset.
    ///
    /// # Errors
    ///
    /// [`ProviderError::Revert`] on contract reverts, [`ProviderError::Rpc`] on transport failures.
    pub async fn get_agent_wallet(&self, agent_id: U256) -> Result<Option<Address>, ProviderError> {
        let wallet = self
            .call(IIdentityRegistry::getAgentWalletCall { agentId: agent_id })
            .await?;
        Ok((wallet != Address::ZERO).then_some(wallet))
    }

    /// `getMetadata`.
    ///
    /// # Errors
    ///
    /// [`ProviderError::Revert`] on contract reverts, [`ProviderError::Rpc`] on transport failures.
    pub async fn get_metadata(&self, agent_id: U256, key: &str) -> Result<Bytes, ProviderError> {
        self.call(IIdentityRegistry::getMetadataCall {
            agentId: agent_id,
            metadataKey: key.into(),
        })
        .await
    }

    /// `isAuthorizedOrOwner`.
    ///
    /// # Errors
    ///
    /// [`ProviderError::Revert`] on contract reverts, [`ProviderError::Rpc`] on transport failures.
    pub async fn is_authorized_or_owner(
        &self,
        spender: Address,
        agent_id: U256,
    ) -> Result<bool, ProviderError> {
        self.call(IIdentityRegistry::isAuthorizedOrOwnerCall {
            spender,
            agentId: agent_id,
        })
        .await
    }

    /// `getApproved`.
    ///
    /// # Errors
    ///
    /// [`ProviderError::Revert`] on contract reverts, [`ProviderError::Rpc`] on transport failures.
    pub async fn get_approved(&self, agent_id: U256) -> Result<Address, ProviderError> {
        self.call(IIdentityRegistry::getApprovedCall { tokenId: agent_id })
            .await
    }

    /// `isApprovedForAll`.
    ///
    /// # Errors
    ///
    /// [`ProviderError::Revert`] on contract reverts, [`ProviderError::Rpc`] on transport failures.
    pub async fn is_approved_for_all(
        &self,
        owner: Address,
        operator: Address,
    ) -> Result<bool, ProviderError> {
        self.call(IIdentityRegistry::isApprovedForAllCall { owner, operator })
            .await
    }

    /// `getVersion`.
    ///
    /// # Errors
    ///
    /// [`ProviderError::Revert`] on contract reverts, [`ProviderError::Rpc`] on transport failures.
    pub async fn get_version(&self) -> Result<String, ProviderError> {
        self.call(IIdentityRegistry::getVersionCall {}).await
    }

    fn check_metadata_key(key: &str) -> Result<(), ProviderError> {
        if key == "agentWallet" {
            return Err(ProviderError::ReservedMetadataKey);
        }
        Ok(())
    }

    /// `register(string[, metadata])` — the agent id comes from the receipt's `Registered` event.
    ///
    /// # Errors
    ///
    /// [`ProviderError::ReservedMetadataKey`] for key `agentWallet`; [`ProviderError::Revert`] on contract reverts;
    /// [`ProviderError::TransactionFailed`] on a failed receipt; [`ProviderError::MissingEvent`] when the
    /// `Registered` event is absent.
    pub async fn register_agent(
        &self,
        agent_uri: &str,
        metadata: &[MetadataEntry],
    ) -> Result<TxOutcome<U256>, ProviderError> {
        for entry in metadata {
            Self::check_metadata_key(&entry.key)?;
        }
        let receipt = if metadata.is_empty() {
            self.send(IIdentityRegistry::register_0Call {
                agentURI: agent_uri.into(),
            })
            .await?
        } else {
            let entries = metadata
                .iter()
                .map(|e| (e.key.clone(), e.value.clone()))
                .collect();
            self.send(IIdentityRegistry::register_2Call {
                agentURI: agent_uri.into(),
                metadata: entries,
            })
            .await?
        };
        let agent_id = parse_registered_agent_id(&receipt)?;
        Ok(TxOutcome::new(&receipt, agent_id))
    }

    /// `setAgentURI`.
    ///
    /// # Errors
    ///
    /// [`ProviderError::Revert`] on contract reverts, [`ProviderError::TransactionFailed`] on a failed receipt.
    pub async fn set_agent_uri(
        &self,
        agent_id: U256,
        agent_uri: &str,
    ) -> Result<TxOutcome<()>, ProviderError> {
        let receipt = self
            .send(IIdentityRegistry::setAgentURICall {
                agentId: agent_id,
                newURI: agent_uri.into(),
            })
            .await?;
        Ok(TxOutcome::new(&receipt, ()))
    }

    /// `setMetadata` — `agentWallet` is reserved.
    ///
    /// # Errors
    ///
    /// [`ProviderError::ReservedMetadataKey`] for key `agentWallet`; [`ProviderError::Revert`] on contract reverts.
    pub async fn set_metadata(
        &self,
        agent_id: U256,
        key: &str,
        value: Bytes,
    ) -> Result<TxOutcome<()>, ProviderError> {
        Self::check_metadata_key(key)?;
        let receipt = self
            .send(IIdentityRegistry::setMetadataCall {
                agentId: agent_id,
                metadataKey: key.into(),
                metadataValue: value,
            })
            .await?;
        Ok(TxOutcome::new(&receipt, ()))
    }

    /// `setAgentWallet` — `deadline` checked against the latest block timestamp (±5 minutes).
    ///
    /// # Errors
    ///
    /// [`ProviderError::Deadline`] on a stale or far-future deadline; [`ProviderError::Revert`] on contract reverts.
    pub async fn set_agent_wallet(
        &self,
        agent_id: U256,
        new_wallet: Address,
        deadline: U256,
        signature: Bytes,
    ) -> Result<TxOutcome<()>, ProviderError> {
        let now = self.client.latest_timestamp().await?;
        let deadline_u64: u64 = deadline.try_into().map_err(|_| {
            ProviderError::Deadline(AgentWalletDeadlineError::DeadlineTooFar {
                deadline: u64::MAX,
                now,
            })
        })?;
        check_agent_wallet_deadline(deadline_u64, now).map_err(ProviderError::Deadline)?;
        let receipt = self
            .send(IIdentityRegistry::setAgentWalletCall {
                agentId: agent_id,
                newWallet: new_wallet,
                deadline,
                signature,
            })
            .await?;
        Ok(TxOutcome::new(&receipt, ()))
    }

    /// `unsetAgentWallet`.
    ///
    /// # Errors
    ///
    /// [`ProviderError::Revert`] on contract reverts, [`ProviderError::TransactionFailed`] on a failed receipt.
    pub async fn unset_agent_wallet(&self, agent_id: U256) -> Result<TxOutcome<()>, ProviderError> {
        let receipt = self
            .send(IIdentityRegistry::unsetAgentWalletCall { agentId: agent_id })
            .await?;
        Ok(TxOutcome::new(&receipt, ()))
    }

    /// `transferFrom` — `from` is read from `ownerOf` (works for owner and
    /// approved-operator senders alike).
    ///
    /// # Errors
    ///
    /// [`ProviderError::Revert`] on contract reverts (e.g. [`crate::revert::RevertReason::IncorrectOwner`]).
    pub async fn transfer_agent(
        &self,
        to: Address,
        agent_id: U256,
    ) -> Result<TxOutcome<()>, ProviderError> {
        let from = self
            .call(IIdentityRegistry::ownerOfCall { agentId: agent_id })
            .await?;
        let receipt = self
            .send(IIdentityRegistry::transferFromCall {
                from,
                to,
                tokenId: agent_id,
            })
            .await?;
        Ok(TxOutcome::new(&receipt, ()))
    }

    /// `approve`.
    ///
    /// # Errors
    ///
    /// [`ProviderError::Revert`] on contract reverts, [`ProviderError::TransactionFailed`] on a failed receipt.
    pub async fn approve_agent(
        &self,
        to: Address,
        agent_id: U256,
    ) -> Result<TxOutcome<()>, ProviderError> {
        let receipt = self
            .send(IIdentityRegistry::approveCall {
                to,
                tokenId: agent_id,
            })
            .await?;
        Ok(TxOutcome::new(&receipt, ()))
    }

    /// `setApprovalForAll`.
    ///
    /// # Errors
    ///
    /// [`ProviderError::Revert`] on contract reverts, [`ProviderError::TransactionFailed`] on a failed receipt.
    pub async fn set_agent_operator(
        &self,
        operator: Address,
        approved: bool,
    ) -> Result<TxOutcome<()>, ProviderError> {
        let receipt = self
            .send(IIdentityRegistry::setApprovalForAllCall { operator, approved })
            .await?;
        Ok(TxOutcome::new(&receipt, ()))
    }
}

/// The `Registered` event's `agent_id` from a register receipt;
/// [`ProviderError::MissingEvent`] when absent.
fn parse_registered_agent_id(receipt: &TransactionReceipt) -> Result<U256, ProviderError> {
    for log in receipt.inner.logs() {
        if let Ok(Some(Event::Registered { agent_id, .. })) = Event::decode(&log.inner) {
            return Ok(agent_id);
        }
    }
    Err(ProviderError::MissingEvent {
        event: "Registered",
    })
}
