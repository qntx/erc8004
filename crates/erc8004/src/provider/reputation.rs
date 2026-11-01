//! Reputation-registry actions.

use alloy::primitives::{Address, B256, U256};
use alloy::providers::Provider;
use alloy::rpc::types::TransactionReceipt;

use crate::abi::IReputationRegistry;
use crate::events::Event;
use crate::feedback_value::FeedbackValue;

use super::{Erc8004, ProviderError, TxOutcome};

/// One `(value, decimals)` feedback row from `readFeedback`.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct Feedback {
    /// The `(value, valueDecimals)` pair.
    pub value: FeedbackValue,
    /// Primary tag.
    pub tag1: String,
    /// Secondary tag.
    pub tag2: String,
    /// Whether the feedback has been revoked.
    pub is_revoked: bool,
}

/// One zipped entry from `readAllFeedback`.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct FeedbackEntry {
    /// The feedback author's address.
    pub client_address: Address,
    /// The feedback index.
    pub feedback_index: u64,
    /// The `(value, valueDecimals)` pair.
    pub value: FeedbackValue,
    /// Primary tag.
    pub tag1: String,
    /// Secondary tag.
    pub tag2: String,
    /// Whether the feedback has been revoked.
    pub is_revoked: bool,
}

/// `getSummary` result.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct ReputationSummary {
    /// Number of matching feedback entries.
    pub count: u64,
    /// The aggregated `(value, valueDecimals)` pair.
    pub value: FeedbackValue,
}

/// `giveFeedback` request — required fields in [`NewFeedback::new`], optional
/// fields via `#[must_use]` chainable setters.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct NewFeedback {
    /// The agent receiving feedback.
    pub agent_id: U256,
    /// The `(value, valueDecimals)` score.
    pub value: FeedbackValue,
    /// Optional primary tag.
    pub tag1: String,
    /// Optional secondary tag.
    pub tag2: String,
    /// Optional endpoint the feedback refers to.
    pub endpoint: String,
    /// Optional off-chain feedback-file URI.
    pub feedback_uri: String,
    /// Optional hash of the feedback file.
    pub feedback_hash: B256,
}

impl NewFeedback {
    /// Builds a request with the required fields; optionals default to empty/`ZERO`.
    #[must_use]
    pub const fn new(agent_id: U256, value: FeedbackValue) -> Self {
        Self {
            agent_id,
            value,
            tag1: String::new(),
            tag2: String::new(),
            endpoint: String::new(),
            feedback_uri: String::new(),
            feedback_hash: B256::ZERO,
        }
    }

    /// Sets the primary tag.
    #[must_use]
    pub fn tag1(mut self, tag1: impl Into<String>) -> Self {
        self.tag1 = tag1.into();
        self
    }

    /// Sets the secondary tag.
    #[must_use]
    pub fn tag2(mut self, tag2: impl Into<String>) -> Self {
        self.tag2 = tag2.into();
        self
    }

    /// Sets the endpoint the feedback refers to.
    #[must_use]
    pub fn endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.endpoint = endpoint.into();
        self
    }

    /// Sets the off-chain feedback-file URI.
    #[must_use]
    pub fn feedback_uri(mut self, feedback_uri: impl Into<String>) -> Self {
        self.feedback_uri = feedback_uri.into();
        self
    }

    /// Sets the feedback-file hash.
    #[must_use]
    pub const fn feedback_hash(mut self, feedback_hash: B256) -> Self {
        self.feedback_hash = feedback_hash;
        self
    }
}

/// Client/tag filter shared by `readAllFeedback` and `getSummary` — empty
/// `clients` means every client (the contract's semantics); `getSummary`
/// additionally requires a non-empty list.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct FeedbackFilter {
    /// Client addresses to filter by; empty matches every client.
    pub client_addresses: Vec<Address>,
    /// Optional primary tag filter.
    pub tag1: String,
    /// Optional secondary tag filter.
    pub tag2: String,
}

impl FeedbackFilter {
    /// Filters to specific clients; `FeedbackFilter::default()` matches every client.
    #[must_use]
    pub fn new(client_addresses: impl Into<Vec<Address>>) -> Self {
        Self {
            client_addresses: client_addresses.into(),
            ..Self::default()
        }
    }

    /// Sets the primary tag filter.
    #[must_use]
    pub fn tag1(mut self, tag1: impl Into<String>) -> Self {
        self.tag1 = tag1.into();
        self
    }

    /// Sets the secondary tag filter.
    #[must_use]
    pub fn tag2(mut self, tag2: impl Into<String>) -> Self {
        self.tag2 = tag2.into();
        self
    }
}

/// Reputation-registry handle borrowed from [`Erc8004`].
#[derive(Debug)]
pub struct Reputation<'a, P> {
    client: &'a Erc8004<P>,
}

impl<'a, P: Provider> Reputation<'a, P> {
    pub(crate) const fn new(client: &'a Erc8004<P>) -> Self {
        Self { client }
    }

    const fn registry(&self) -> Address {
        self.client.deployment().reputation_registry
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

    /// `readFeedback`.
    ///
    /// # Errors
    ///
    /// [`ProviderError::Revert`] on contract reverts, [`ProviderError::Rpc`] on transport failures,
    /// [`ProviderError::InvalidResponse`] when the on-chain value violates [`FeedbackValue`] bounds.
    pub async fn read_feedback(
        &self,
        agent_id: U256,
        client_address: Address,
        feedback_index: u64,
    ) -> Result<Feedback, ProviderError> {
        let result = self
            .call(IReputationRegistry::readFeedbackCall {
                agentId: agent_id,
                clientAddress: client_address,
                feedbackIndex: feedback_index,
            })
            .await?;
        let is_revoked = result.isRevoked;
        let pair = FeedbackValue::new(result.value, result.valueDecimals)
            .map_err(|e| ProviderError::InvalidResponse(e.to_string()))?;
        Ok(Feedback {
            value: pair,
            tag1: result.tag1,
            tag2: result.tag2,
            is_revoked,
        })
    }

    /// `readAllFeedback` — parallel arrays zipped into [`FeedbackEntry`]s.
    ///
    /// # Errors
    ///
    /// [`ProviderError::Revert`] on contract reverts, [`ProviderError::Rpc`] on transport failures.
    pub async fn read_all_feedback(
        &self,
        agent_id: U256,
        filter: &FeedbackFilter,
        include_revoked: bool,
    ) -> Result<Vec<FeedbackEntry>, ProviderError> {
        let result = self
            .call(IReputationRegistry::readAllFeedbackCall {
                agentId: agent_id,
                clientAddresses: filter.client_addresses.clone(),
                tag1: filter.tag1.clone(),
                tag2: filter.tag2.clone(),
                includeRevoked: include_revoked,
            })
            .await?;
        let IReputationRegistry::readAllFeedbackReturn {
            clients,
            feedbackIndexes: indexes,
            values,
            valueDecimals: decimals,
            tag1s,
            tag2s,
            revokedStatuses: revoked,
        } = result;
        let n = clients.len();
        if [
            indexes.len(),
            values.len(),
            decimals.len(),
            tag1s.len(),
            tag2s.len(),
            revoked.len(),
        ]
        .iter()
        .any(|&len| len != n)
        {
            return Err(ProviderError::InvalidResponse(
                "readAllFeedback returned mismatched array lengths".into(),
            ));
        }
        clients
            .into_iter()
            .zip(indexes)
            .zip(values.into_iter().zip(decimals))
            .zip(tag1s.into_iter().zip(tag2s))
            .zip(revoked)
            .map(
                |(
                    (((client_address, feedback_index), (value, decimals)), (tag1, tag2)),
                    is_revoked,
                )| {
                    Ok(FeedbackEntry {
                        client_address,
                        feedback_index,
                        value: FeedbackValue::new(value, decimals)
                            .map_err(|e| ProviderError::InvalidResponse(e.to_string()))?,
                        tag1,
                        tag2,
                        is_revoked,
                    })
                },
            )
            .collect()
    }

    /// `getSummary` — `filter.client_addresses` must be non-empty.
    ///
    /// # Errors
    ///
    /// [`ProviderError::ClientAddressesRequired`] for an empty client list; [`ProviderError::Revert`] on
    /// contract reverts.
    pub async fn get_reputation_summary(
        &self,
        agent_id: U256,
        filter: &FeedbackFilter,
    ) -> Result<ReputationSummary, ProviderError> {
        if filter.client_addresses.is_empty() {
            return Err(ProviderError::ClientAddressesRequired);
        }
        let result = self
            .call(IReputationRegistry::getSummaryCall {
                agentId: agent_id,
                clientAddresses: filter.client_addresses.clone(),
                tag1: filter.tag1.clone(),
                tag2: filter.tag2.clone(),
            })
            .await?;
        let count = result.count;
        let pair = FeedbackValue::new(result.summaryValue, result.summaryValueDecimals)
            .map_err(|e| ProviderError::InvalidResponse(e.to_string()))?;
        Ok(ReputationSummary { count, value: pair })
    }

    /// `getClients`.
    ///
    /// # Errors
    ///
    /// [`ProviderError::Revert`] on contract reverts, [`ProviderError::Rpc`] on transport failures.
    pub async fn get_clients(&self, agent_id: U256) -> Result<Vec<Address>, ProviderError> {
        self.call(IReputationRegistry::getClientsCall { agentId: agent_id })
            .await
    }

    /// `getLastIndex`.
    ///
    /// # Errors
    ///
    /// [`ProviderError::Revert`] on contract reverts, [`ProviderError::Rpc`] on transport failures.
    pub async fn get_last_index(
        &self,
        agent_id: U256,
        client_address: Address,
    ) -> Result<u64, ProviderError> {
        self.call(IReputationRegistry::getLastIndexCall {
            agentId: agent_id,
            clientAddress: client_address,
        })
        .await
    }

    /// `getResponseCount` — contract semantics: zero `client_address` → every client;
    /// `feedback_index` 0 → every feedback of that client; empty `responders` → every responder.
    ///
    /// # Errors
    ///
    /// [`ProviderError::Revert`] on contract reverts, [`ProviderError::Rpc`] on transport failures.
    pub async fn get_response_count(
        &self,
        agent_id: U256,
        client_address: Option<Address>,
        feedback_index: Option<u64>,
        responders: &[Address],
    ) -> Result<u64, ProviderError> {
        self.call(IReputationRegistry::getResponseCountCall {
            agentId: agent_id,
            clientAddress: client_address.unwrap_or(Address::ZERO),
            feedbackIndex: feedback_index.unwrap_or(0),
            responders: responders.to_vec(),
        })
        .await
    }

    /// `getVersion`.
    ///
    /// # Errors
    ///
    /// [`ProviderError::Revert`] on contract reverts, [`ProviderError::Rpc`] on transport failures.
    pub async fn get_version(&self) -> Result<String, ProviderError> {
        self.call(IReputationRegistry::getVersionCall {}).await
    }

    /// `giveFeedback` — the index comes from the receipt's `NewFeedback` event.
    ///
    /// # Errors
    ///
    /// [`ProviderError::Revert`] on contract reverts (e.g. [`crate::revert::RevertReason::SelfFeedback`]);
    /// [`ProviderError::MissingEvent`] when `NewFeedback` is absent.
    pub async fn give_feedback(
        &self,
        feedback: &NewFeedback,
    ) -> Result<TxOutcome<u64>, ProviderError> {
        let receipt = self
            .send(IReputationRegistry::giveFeedbackCall {
                agentId: feedback.agent_id,
                value: feedback.value.value(),
                valueDecimals: feedback.value.decimals(),
                tag1: feedback.tag1.clone(),
                tag2: feedback.tag2.clone(),
                endpoint: feedback.endpoint.clone(),
                feedbackURI: feedback.feedback_uri.clone(),
                feedbackHash: feedback.feedback_hash,
            })
            .await?;
        let index = parse_feedback_index(&receipt)?;
        Ok(TxOutcome::new(&receipt, index))
    }

    /// `revokeFeedback`.
    ///
    /// # Errors
    ///
    /// [`ProviderError::Revert`] on contract reverts (e.g. [`crate::revert::RevertReason::AlreadyRevoked`]).
    pub async fn revoke_feedback(
        &self,
        agent_id: U256,
        feedback_index: u64,
    ) -> Result<TxOutcome<()>, ProviderError> {
        let receipt = self
            .send(IReputationRegistry::revokeFeedbackCall {
                agentId: agent_id,
                feedbackIndex: feedback_index,
            })
            .await?;
        Ok(TxOutcome::new(&receipt, ()))
    }

    /// `appendResponse` — `response_uri` must be non-empty.
    ///
    /// # Errors
    ///
    /// [`ProviderError::EmptyResponseUri`] for an empty URI; [`ProviderError::Revert`] on contract reverts.
    pub async fn append_response(
        &self,
        agent_id: U256,
        client_address: Address,
        feedback_index: u64,
        response_uri: &str,
        response_hash: B256,
    ) -> Result<TxOutcome<()>, ProviderError> {
        if response_uri.is_empty() {
            return Err(ProviderError::EmptyResponseUri);
        }
        let receipt = self
            .send(IReputationRegistry::appendResponseCall {
                agentId: agent_id,
                clientAddress: client_address,
                feedbackIndex: feedback_index,
                responseURI: response_uri.into(),
                responseHash: response_hash,
            })
            .await?;
        Ok(TxOutcome::new(&receipt, ()))
    }
}

/// The `NewFeedback` event's `feedback_index` from a `giveFeedback` receipt.
fn parse_feedback_index(receipt: &TransactionReceipt) -> Result<u64, ProviderError> {
    for log in receipt.inner.logs() {
        if let Ok(Some(Event::NewFeedback { feedback_index, .. })) = Event::decode(&log.inner) {
            return Ok(feedback_index);
        }
    }
    Err(ProviderError::MissingEvent {
        event: "NewFeedback",
    })
}
