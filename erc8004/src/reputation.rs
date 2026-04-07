//! Reputation Registry operations.
//!
//! The Reputation Registry provides a standard interface for posting and
//! fetching feedback signals about agents. Scoring uses `int128` values
//! with configurable decimal places, and feedback can be categorized
//! with two tags and an endpoint reference.
//!
//! This module wraps all read and write functions exposed by the
//! `ReputationRegistryUpgradeable` contract.

use alloy::{
    primitives::{Address, FixedBytes, U256},
    providers::Provider,
};

use crate::{
    contracts::ReputationRegistry,
    error::Result,
    types::{Feedback, FeedbackInput, ReputationSummary},
};

/// A handle to the Reputation Registry contract bound to a specific provider.
///
/// Created via [`Erc8004::reputation()`](crate::Erc8004::reputation).
#[derive(Debug)]
pub struct Reputation<P> {
    address: Address,
    provider: P,
}

impl<P: Provider> Reputation<P> {
    /// Create a new `Reputation` handle from a provider and contract address.
    pub(crate) const fn new(provider: P, address: Address) -> Self {
        Self { address, provider }
    }

    /// Submit feedback for an agent.
    ///
    /// See [`FeedbackInput`] for field documentation.
    ///
    /// # Errors
    ///
    /// Returns an error if the transaction fails.
    pub async fn give_feedback(&self, input: FeedbackInput) -> Result<()> {
        let contract = ReputationRegistry::new(self.address, &self.provider);
        contract
            .giveFeedback(
                input.agent_id,
                input.value,
                input.value_decimals,
                input.tag1,
                input.tag2,
                input.endpoint,
                input.feedback_uri,
                input.feedback_hash,
            )
            .send()
            .await?
            .get_receipt()
            .await?;
        Ok(())
    }

    /// Revoke previously submitted feedback.
    ///
    /// Must be called by the original `clientAddress`.
    ///
    /// # Errors
    ///
    /// Returns an error if the transaction fails.
    pub async fn revoke_feedback(&self, agent_id: U256, feedback_index: u64) -> Result<()> {
        let contract = ReputationRegistry::new(self.address, &self.provider);
        contract
            .revokeFeedback(agent_id, feedback_index)
            .send()
            .await?
            .get_receipt()
            .await?;
        Ok(())
    }

    /// Append a response to existing feedback.
    ///
    /// Anyone may call this (e.g. the agent showing a refund, or a
    /// data intelligence aggregator tagging feedback as spam).
    ///
    /// # Errors
    ///
    /// Returns an error if the transaction fails.
    pub async fn append_response(
        &self,
        agent_id: U256,
        client_address: Address,
        feedback_index: u64,
        response_uri: &str,
        response_hash: FixedBytes<32>,
    ) -> Result<()> {
        let contract = ReputationRegistry::new(self.address, &self.provider);
        contract
            .appendResponse(
                agent_id,
                client_address,
                feedback_index,
                response_uri.to_owned(),
                response_hash,
            )
            .send()
            .await?
            .get_receipt()
            .await?;
        Ok(())
    }

    /// Read a single feedback entry.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails.
    pub async fn read_feedback(
        &self,
        agent_id: U256,
        client_address: Address,
        feedback_index: u64,
    ) -> Result<Feedback> {
        let contract = ReputationRegistry::new(self.address, &self.provider);
        let r = contract
            .readFeedback(agent_id, client_address, feedback_index)
            .call()
            .await?;
        Ok(Feedback {
            value: r.value,
            value_decimals: r.valueDecimals,
            tag1: r.tag1,
            tag2: r.tag2,
            is_revoked: r.isRevoked,
        })
    }

    /// Read all feedback for an agent with optional filters.
    ///
    /// Returns the raw return struct from the contract.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails.
    pub async fn read_all_feedback(
        &self,
        agent_id: U256,
        client_addresses: Vec<Address>,
        tag1: &str,
        tag2: &str,
        include_revoked: bool,
    ) -> Result<ReputationRegistry::readAllFeedbackReturn> {
        let contract = ReputationRegistry::new(self.address, &self.provider);
        Ok(contract
            .readAllFeedback(
                agent_id,
                client_addresses,
                tag1.to_owned(),
                tag2.to_owned(),
                include_revoked,
            )
            .call()
            .await?)
    }

    /// Get an aggregated reputation summary for an agent.
    ///
    /// **Important**: `client_addresses` MUST be non-empty to avoid
    /// Sybil/spam attacks (as per the ERC-8004 specification).
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails.
    pub async fn get_summary(
        &self,
        agent_id: U256,
        client_addresses: Vec<Address>,
        tag1: &str,
        tag2: &str,
    ) -> Result<ReputationSummary> {
        let contract = ReputationRegistry::new(self.address, &self.provider);
        let r = contract
            .getSummary(agent_id, client_addresses, tag1.to_owned(), tag2.to_owned())
            .call()
            .await?;
        Ok(ReputationSummary {
            count: r.count,
            summary_value: r.summaryValue,
            summary_value_decimals: r.summaryValueDecimals,
        })
    }

    /// Get all client addresses that have submitted feedback for an agent.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails.
    pub async fn get_clients(&self, agent_id: U256) -> Result<Vec<Address>> {
        let contract = ReputationRegistry::new(self.address, &self.provider);
        Ok(contract.getClients(agent_id).call().await?)
    }

    /// Get the last feedback index for a specific client-agent pair.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails.
    pub async fn get_last_index(&self, agent_id: U256, client_address: Address) -> Result<u64> {
        let contract = ReputationRegistry::new(self.address, &self.provider);
        Ok(contract
            .getLastIndex(agent_id, client_address)
            .call()
            .await?)
    }

    /// Get the number of responses appended to a specific feedback entry.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails.
    pub async fn get_response_count(
        &self,
        agent_id: U256,
        client_address: Address,
        feedback_index: u64,
        responders: Vec<Address>,
    ) -> Result<u64> {
        let contract = ReputationRegistry::new(self.address, &self.provider);
        Ok(contract
            .getResponseCount(agent_id, client_address, feedback_index, responders)
            .call()
            .await?)
    }

    /// Get the address of the linked Identity Registry.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails.
    pub async fn get_identity_registry(&self) -> Result<Address> {
        let contract = ReputationRegistry::new(self.address, &self.provider);
        Ok(contract.getIdentityRegistry().call().await?)
    }

    /// Get the contract version string.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails.
    pub async fn get_version(&self) -> Result<String> {
        let contract = ReputationRegistry::new(self.address, &self.provider);
        Ok(contract.getVersion().call().await?)
    }
}
