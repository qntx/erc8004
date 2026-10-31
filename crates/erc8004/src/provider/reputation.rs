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
    pub client: Address,
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
        client: Address,
        feedback_index: u64,
    ) -> Result<Feedback, ProviderError> {
        let result = self
            .call(IReputationRegistry::readFeedbackCall {
                agentId: agent_id,
                clientAddress: client,
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
        clients: &[Address],
        tag1: &str,
        tag2: &str,
        include_revoked: bool,
    ) -> Result<Vec<FeedbackEntry>, ProviderError> {
        let result = self
            .call(IReputationRegistry::readAllFeedbackCall {
                agentId: agent_id,
                clientAddresses: clients.to_vec(),
                tag1: tag1.into(),
                tag2: tag2.into(),
                includeRevoked: include_revoked,
            })
            .await?;
        let clients_r = result.clients;
        let indexes = result.feedbackIndexes;
        let values = result.values;
        let decimals = result.valueDecimals;
        let tag1s = result.tag1s;
        let tag2s = result.tag2s;
        let revoked = result.revokedStatuses;
        let mut out = Vec::with_capacity(clients_r.len());
        for (i, client) in clients_r.iter().enumerate() {
            let pair =
                FeedbackValue::new(*values.get(i).unwrap_or(&0), *decimals.get(i).unwrap_or(&0))
                    .map_err(|e| ProviderError::InvalidResponse(e.to_string()))?;
            out.push(FeedbackEntry {
                client: *client,
                feedback_index: *indexes.get(i).unwrap_or(&0),
                value: pair,
                tag1: tag1s.get(i).cloned().unwrap_or_default(),
                tag2: tag2s.get(i).cloned().unwrap_or_default(),
                is_revoked: *revoked.get(i).unwrap_or(&false),
            });
        }
        Ok(out)
    }

    /// `getSummary` — `clients` must be non-empty.
    ///
    /// # Errors
    ///
    /// [`ProviderError::ClientAddressesRequired`] for an empty client list; [`ProviderError::Revert`] on
    /// contract reverts.
    pub async fn get_reputation_summary(
        &self,
        agent_id: U256,
        clients: &[Address],
        tag1: &str,
        tag2: &str,
    ) -> Result<ReputationSummary, ProviderError> {
        if clients.is_empty() {
            return Err(ProviderError::ClientAddressesRequired);
        }
        let result = self
            .call(IReputationRegistry::getSummaryCall {
                agentId: agent_id,
                clientAddresses: clients.to_vec(),
                tag1: tag1.into(),
                tag2: tag2.into(),
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
        client: Address,
    ) -> Result<u64, ProviderError> {
        self.call(IReputationRegistry::getLastIndexCall {
            agentId: agent_id,
            clientAddress: client,
        })
        .await
    }

    /// `getResponseCount` — `client`/`feedback_index`/`responders` are paired (all set or all unset).
    ///
    /// # Errors
    ///
    /// [`ProviderError::Revert`] on contract reverts, [`ProviderError::Rpc`] on transport failures.
    pub async fn get_response_count(
        &self,
        agent_id: U256,
        client: Option<Address>,
        feedback_index: Option<u64>,
        responders: Option<Vec<Address>>,
    ) -> Result<u64, ProviderError> {
        self.call(IReputationRegistry::getResponseCountCall {
            agentId: agent_id,
            clientAddress: client.unwrap_or(Address::ZERO),
            feedbackIndex: feedback_index.unwrap_or(0),
            responders: responders.unwrap_or_default(),
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
    #[allow(
        clippy::too_many_arguments,
        reason = "contract ABI signature matches upstream"
    )]
    ///
    /// # Errors
    ///
    /// [`ProviderError::Revert`] on contract reverts (e.g. [`crate::revert::RevertReason::SelfFeedback`]);
    /// [`ProviderError::MissingEvent`] when `NewFeedback` is absent.
    pub async fn give_feedback(
        &self,
        agent_id: U256,
        value: FeedbackValue,
        tag1: &str,
        tag2: &str,
        endpoint: &str,
        feedback_uri: &str,
        feedback_hash: B256,
    ) -> Result<TxOutcome<u64>, ProviderError> {
        let receipt = self
            .send(IReputationRegistry::giveFeedbackCall {
                agentId: agent_id,
                value: value.value(),
                valueDecimals: value.decimals(),
                tag1: tag1.into(),
                tag2: tag2.into(),
                endpoint: endpoint.into(),
                feedbackURI: feedback_uri.into(),
                feedbackHash: feedback_hash,
            })
            .await?;
        let index = parse_feedback_index(&receipt)?;
        Ok(TxOutcome {
            tx_hash: receipt.transaction_hash,
            value: index,
        })
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
        Ok(TxOutcome {
            tx_hash: receipt.transaction_hash,
            value: (),
        })
    }

    /// `appendResponse` — `response_uri` must be non-empty.
    ///
    /// # Errors
    ///
    /// [`ProviderError::EmptyResponseUri`] for an empty URI; [`ProviderError::Revert`] on contract reverts.
    pub async fn append_response(
        &self,
        agent_id: U256,
        client: Address,
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
                clientAddress: client,
                feedbackIndex: feedback_index,
                responseURI: response_uri.into(),
                responseHash: response_hash,
            })
            .await?;
        Ok(TxOutcome {
            tx_hash: receipt.transaction_hash,
            value: (),
        })
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
