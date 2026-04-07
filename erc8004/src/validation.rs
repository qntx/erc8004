//! Validation Registry operations.
//!
//! The Validation Registry enables agents to request independent verification
//! of their work (e.g. stake-secured re-execution, zkML proofs, TEE oracles)
//! and allows validator contracts to record responses on-chain.
//!
//! This module wraps all read and write functions exposed by the
//! `ValidationRegistryUpgradeable` contract.

use alloy::{
    primitives::{Address, FixedBytes, U256},
    providers::Provider,
};

use crate::{
    contracts::ValidationRegistry,
    error::Result,
    types::{ValidationStatus, ValidationSummary},
};

/// A handle to the Validation Registry contract bound to a specific provider.
///
/// Created via [`Erc8004::validation()`](crate::Erc8004::validation).
#[derive(Debug)]
pub struct Validation<P> {
    address: Address,
    provider: P,
}

impl<P: Provider> Validation<P> {
    /// Create a new `Validation` handle from a provider and contract address.
    pub(crate) const fn new(provider: P, address: Address) -> Self {
        Self { address, provider }
    }

    /// Submit a validation request for an agent.
    ///
    /// Must be called by the owner or operator of `agent_id`.
    ///
    /// # Parameters
    ///
    /// - `validator_address`: The validator contract / EOA to handle the request.
    /// - `agent_id`: The agent requesting validation.
    /// - `request_uri`: URI pointing to off-chain data needed for validation.
    /// - `request_hash`: Keccak-256 hash of the request payload (commitment).
    ///
    /// # Errors
    ///
    /// Returns an error if the transaction fails.
    pub async fn submit_request(
        &self,
        validator_address: Address,
        agent_id: U256,
        request_uri: &str,
        request_hash: FixedBytes<32>,
    ) -> Result<()> {
        let contract = ValidationRegistry::new(self.address, &self.provider);
        contract
            .validationRequest(
                validator_address,
                agent_id,
                request_uri.to_owned(),
                request_hash,
            )
            .send()
            .await?
            .get_receipt()
            .await?;
        Ok(())
    }

    /// Submit a validation response.
    ///
    /// Must be called by the `validatorAddress` specified in the original
    /// request. Can be called multiple times for progressive validation.
    ///
    /// # Parameters
    ///
    /// - `request_hash`: The hash identifying the original request.
    /// - `response`: A value 0-100 (0 = failed, 100 = passed).
    /// - `response_uri`: Optional URI pointing to validation evidence.
    /// - `response_hash`: Optional hash of the response content.
    /// - `tag`: Optional categorization tag.
    ///
    /// # Errors
    ///
    /// Returns an error if the transaction fails.
    pub async fn submit_response(
        &self,
        request_hash: FixedBytes<32>,
        response: u8,
        response_uri: &str,
        response_hash: FixedBytes<32>,
        tag: &str,
    ) -> Result<()> {
        let contract = ValidationRegistry::new(self.address, &self.provider);
        contract
            .validationResponse(
                request_hash,
                response,
                response_uri.to_owned(),
                response_hash,
                tag.to_owned(),
            )
            .send()
            .await?
            .get_receipt()
            .await?;
        Ok(())
    }

    /// Get the current status of a validation request.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails.
    pub async fn get_validation_status(
        &self,
        request_hash: FixedBytes<32>,
    ) -> Result<ValidationStatus> {
        let contract = ValidationRegistry::new(self.address, &self.provider);
        let r = contract.getValidationStatus(request_hash).call().await?;
        Ok(ValidationStatus {
            validator_address: r.validatorAddress,
            agent_id: r.agentId,
            response: r.response,
            response_hash: r.responseHash,
            tag: r.tag,
            last_update: r.lastUpdate,
        })
    }

    /// Get an aggregated validation summary for an agent.
    ///
    /// # Parameters
    ///
    /// - `agent_id`: The target agent (mandatory).
    /// - `validator_addresses`: Filter by specific validators (empty = all).
    /// - `tag`: Filter by tag (empty = no filter).
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails.
    pub async fn get_summary(
        &self,
        agent_id: U256,
        validator_addresses: &[Address],
        tag: &str,
    ) -> Result<ValidationSummary> {
        let contract = ValidationRegistry::new(self.address, &self.provider);
        let r = contract
            .getSummary(agent_id, validator_addresses.to_vec(), tag.to_owned())
            .call()
            .await?;
        Ok(ValidationSummary {
            count: r.count,
            avg_response: r.avgResponse,
        })
    }

    /// Get all validation request hashes for an agent.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails.
    pub async fn get_agent_validations(&self, agent_id: U256) -> Result<Vec<FixedBytes<32>>> {
        let contract = ValidationRegistry::new(self.address, &self.provider);
        Ok(contract.getAgentValidations(agent_id).call().await?)
    }

    /// Get all validation request hashes assigned to a validator.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails.
    pub async fn get_validator_requests(
        &self,
        validator_address: Address,
    ) -> Result<Vec<FixedBytes<32>>> {
        let contract = ValidationRegistry::new(self.address, &self.provider);
        Ok(contract
            .getValidatorRequests(validator_address)
            .call()
            .await?)
    }

    /// Get the address of the linked Identity Registry.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails.
    pub async fn get_identity_registry(&self) -> Result<Address> {
        let contract = ValidationRegistry::new(self.address, &self.provider);
        Ok(contract.getIdentityRegistry().call().await?)
    }

    /// Get the contract version string.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails.
    pub async fn get_version(&self) -> Result<String> {
        let contract = ValidationRegistry::new(self.address, &self.provider);
        Ok(contract.getVersion().call().await?)
    }
}
