//! Validation-registry actions.

use alloy::primitives::{Address, B256, U256};
use alloy::providers::Provider;
use alloy::rpc::types::TransactionReceipt;

use crate::abi::IValidationRegistry;
use crate::revert::RevertReason;

use super::{Erc8004, ProviderError, TxOutcome};

/// `getValidationStatus` row.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ValidationStatus {
    /// The validator's address.
    pub validator: Address,
    /// The agent id.
    pub agent_id: U256,
    /// The response (0..=100).
    pub response: u8,
    /// The response hash.
    pub response_hash: B256,
    /// The tag.
    pub tag: String,
    /// The last update timestamp.
    pub last_update: U256,
}

/// `getSummary` result.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ValidationSummary {
    /// Number of matching requests.
    pub count: u64,
    /// The average response.
    pub average_response: u8,
}

/// `validationResponse` request — required fields in [`ValidationResponse::new`],
/// optional fields via `#[must_use]` chainable setters.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ValidationResponse {
    /// The request being answered.
    pub request_hash: B256,
    /// The response score (`0..=100`).
    pub response: u8,
    /// Optional off-chain response URI.
    pub response_uri: String,
    /// Optional response hash.
    pub response_hash: B256,
    /// Optional tag.
    pub tag: String,
}

impl ValidationResponse {
    /// Builds a request with the required fields; optionals default to empty/`ZERO`.
    #[must_use]
    pub const fn new(request_hash: B256, response: u8) -> Self {
        Self {
            request_hash,
            response,
            response_uri: String::new(),
            response_hash: B256::ZERO,
            tag: String::new(),
        }
    }

    /// Sets the off-chain response URI.
    #[must_use]
    pub fn response_uri(mut self, response_uri: impl Into<String>) -> Self {
        self.response_uri = response_uri.into();
        self
    }

    /// Sets the response hash.
    #[must_use]
    pub const fn response_hash(mut self, response_hash: B256) -> Self {
        self.response_hash = response_hash;
        self
    }

    /// Sets the tag.
    #[must_use]
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tag = tag.into();
        self
    }
}

/// Validation-registry handle borrowed from [`Erc8004`].
#[derive(Debug)]
pub struct Validation<'a, P> {
    client: &'a Erc8004<P>,
}

impl<'a, P: Provider> Validation<'a, P> {
    pub(crate) const fn new(client: &'a Erc8004<P>) -> Self {
        Self { client }
    }

    const fn registry(&self) -> Address {
        self.client.deployment().validation_registry
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

    /// `getValidationStatus` — `None` exactly on an `UnknownRequest` revert.
    ///
    /// # Errors
    ///
    /// [`ProviderError::Revert`] on contract reverts other than `UnknownRequest`,
    /// [`ProviderError::Rpc`] on transport failures.
    pub async fn get_validation_status(
        &self,
        request_hash: B256,
    ) -> Result<Option<ValidationStatus>, ProviderError> {
        match self
            .call(IValidationRegistry::getValidationStatusCall {
                requestHash: request_hash,
            })
            .await
        {
            Ok(r) => Ok(Some(ValidationStatus {
                validator: r.validatorAddress,
                agent_id: r.agentId,
                response: r.response,
                response_hash: r.responseHash,
                tag: r.tag,
                last_update: r.lastUpdate,
            })),
            Err(ProviderError::Revert(RevertReason::UnknownRequest)) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// `getSummary`.
    ///
    /// # Errors
    ///
    /// [`ProviderError::Revert`] on contract reverts, [`ProviderError::Rpc`] on transport failures.
    pub async fn get_validation_summary(
        &self,
        agent_id: U256,
        validators: &[Address],
        tag: &str,
    ) -> Result<ValidationSummary, ProviderError> {
        let result = self
            .call(IValidationRegistry::getSummaryCall {
                agentId: agent_id,
                validatorAddresses: validators.to_vec(),
                tag: tag.into(),
            })
            .await?;
        Ok(ValidationSummary {
            count: result.count,
            average_response: result.avgResponse,
        })
    }

    /// `getAgentValidations`.
    ///
    /// # Errors
    ///
    /// [`ProviderError::Revert`] on contract reverts, [`ProviderError::Rpc`] on transport failures.
    pub async fn get_agent_validations(&self, agent_id: U256) -> Result<Vec<B256>, ProviderError> {
        self.call(IValidationRegistry::getAgentValidationsCall { agentId: agent_id })
            .await
    }

    /// `getValidatorRequests`.
    ///
    /// # Errors
    ///
    /// [`ProviderError::Revert`] on contract reverts, [`ProviderError::Rpc`] on transport failures.
    pub async fn get_validator_requests(
        &self,
        validator: Address,
    ) -> Result<Vec<B256>, ProviderError> {
        self.call(IValidationRegistry::getValidatorRequestsCall {
            validatorAddress: validator,
        })
        .await
    }

    /// `getVersion`.
    ///
    /// # Errors
    ///
    /// [`ProviderError::Revert`] on contract reverts, [`ProviderError::Rpc`] on transport failures.
    pub async fn get_version(&self) -> Result<String, ProviderError> {
        self.call(IValidationRegistry::getVersionCall {}).await
    }

    /// `validationRequest`.
    ///
    /// # Errors
    ///
    /// [`ProviderError::Revert`] on contract reverts (e.g. [`RevertReason::RequestExists`]).
    pub async fn request_validation(
        &self,
        validator: Address,
        agent_id: U256,
        request_uri: &str,
        request_hash: B256,
    ) -> Result<TxOutcome<()>, ProviderError> {
        let receipt = self
            .send(IValidationRegistry::validationRequestCall {
                validatorAddress: validator,
                agentId: agent_id,
                requestURI: request_uri.into(),
                requestHash: request_hash,
            })
            .await?;
        Ok(TxOutcome {
            tx_hash: receipt.transaction_hash,
            value: (),
        })
    }

    /// `validationResponse` — `response` must be `0..=100`.
    ///
    /// # Errors
    ///
    /// [`ProviderError::ResponseOutOfRange`] for out-of-range values; [`ProviderError::Revert`] on
    /// contract reverts (e.g. [`RevertReason::NotValidator`]).
    pub async fn respond_to_validation(
        &self,
        response: &ValidationResponse,
    ) -> Result<TxOutcome<()>, ProviderError> {
        if response.response > 100 {
            return Err(ProviderError::ResponseOutOfRange);
        }
        let receipt = self
            .send(IValidationRegistry::validationResponseCall {
                requestHash: response.request_hash,
                response: response.response,
                responseURI: response.response_uri.clone(),
                responseHash: response.response_hash,
                tag: response.tag.clone(),
            })
            .await?;
        Ok(TxOutcome {
            tx_hash: receipt.transaction_hash,
            value: (),
        })
    }
}
