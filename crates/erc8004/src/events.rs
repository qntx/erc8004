//! Decodes one protocol log by `topic0`: unknown topics return `Ok(None)`;
//! malformed data under a known topic is [`EventDecodeError::MalformedLog`].

/// Contract log decode failure under a known protocol event topic.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum EventDecodeError {
    /// Topics or data are malformed for the event's ABI shape.
    #[error("malformed log: {0}")]
    MalformedLog(String),
}

impl EventDecodeError {
    /// The shared error code.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::MalformedLog(_) => "MalformedLog",
        }
    }
}

use alloy::primitives::{Address, B256, Bytes, Log, U256};
use alloy::sol_types::SolEvent;

use crate::abi::{IIdentityRegistry, IReputationRegistry, IValidationRegistry};
use thiserror::Error;

fn malformed(detail: impl Into<String>) -> EventDecodeError {
    EventDecodeError::MalformedLog(detail.into())
}

/// One decoded protocol event, `snake_case` fields matching the ABI.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum Event {
    /// `Registered(agentId, agentURI, owner)`.
    Registered {
        /// The agent id.
        agent_id: U256,
        /// The `agentURI` value.
        agent_uri: String,
        /// The owner address.
        owner: Address,
    },
    /// `URIUpdated(agentId, newURI, updatedBy)`.
    UriUpdated {
        /// The agent id.
        agent_id: U256,
        /// The updated `agentURI` value.
        new_uri: String,
        /// The caller that updated the URI.
        updated_by: Address,
    },
    /// `MetadataSet(agentId, indexedMetadataKey, metadataKey, metadataValue)`; `indexedMetadataKey` is the topic hash.
    MetadataSet {
        /// The agent id.
        agent_id: U256,
        /// Keccak topic hash of the indexed metadata key.
        indexed_metadata_key: B256,
        /// The metadata key.
        metadata_key: String,
        /// The metadata value.
        metadata_value: Bytes,
    },
    /// `Transfer(from, to, tokenId)`.
    Transfer {
        /// The sender address.
        from: Address,
        /// The recipient address.
        to: Address,
        /// The token id.
        token_id: U256,
    },
    /// `Approval(owner, approved, tokenId)`.
    Approval {
        /// The owner address.
        owner: Address,
        /// The approved address.
        approved: Address,
        /// The token id.
        token_id: U256,
    },
    /// `ApprovalForAll(owner, operator, approved)`.
    ApprovalForAll {
        /// The owner address.
        owner: Address,
        /// The operator address.
        operator: Address,
        /// Whether the operator is approved.
        approved: bool,
    },
    /// `MetadataUpdate(tokenId)`.
    MetadataUpdate {
        /// The token id.
        token_id: U256,
    },
    /// `BatchMetadataUpdate(fromTokenId, toTokenId)`.
    BatchMetadataUpdate {
        /// First token id of the batch range.
        from_token_id: U256,
        /// Last token id of the batch range.
        to_token_id: U256,
    },
    /// `NewFeedback(...)`.
    NewFeedback {
        /// The agent id.
        agent_id: U256,
        /// The client address.
        client_address: Address,
        /// The feedback index.
        feedback_index: u64,
        /// The `int128` feedback value.
        value: i128,
        /// The `uint8` decimals of `value`.
        value_decimals: u8,
        /// Keccak topic hash of `tag1`.
        indexed_tag1: B256,
        /// The primary tag.
        tag1: String,
        /// The secondary tag.
        tag2: String,
        /// The endpoint.
        endpoint: String,
        /// The feedback URI.
        feedback_uri: String,
        /// `keccak256` of the feedback file.
        feedback_hash: B256,
    },
    /// `FeedbackRevoked(agentId, clientAddress, feedbackIndex)`.
    FeedbackRevoked {
        /// The agent id.
        agent_id: U256,
        /// The client address.
        client_address: Address,
        /// The feedback index.
        feedback_index: u64,
    },
    /// `ResponseAppended(...)`.
    ResponseAppended {
        /// The agent id.
        agent_id: U256,
        /// The client address.
        client_address: Address,
        /// The feedback index.
        feedback_index: u64,
        /// The responder address.
        responder: Address,
        /// The response URI.
        response_uri: String,
        /// `keccak256` of the response.
        response_hash: B256,
    },
    /// `ValidationRequest(validatorAddress, agentId, requestURI, requestHash)`.
    ValidationRequest {
        /// The validator address.
        validator_address: Address,
        /// The agent id.
        agent_id: U256,
        /// The request URI.
        request_uri: String,
        /// `keccak256` of the request.
        request_hash: B256,
    },
    /// `ValidationResponse(...)`.
    ValidationResponse {
        /// The validator address.
        validator_address: Address,
        /// The agent id.
        agent_id: U256,
        /// `keccak256` of the request.
        request_hash: B256,
        /// The `uint8` response value.
        response: u8,
        /// The response URI.
        response_uri: String,
        /// `keccak256` of the response.
        response_hash: B256,
        /// The tag.
        tag: String,
    },
}

/// The event name for the shared `{"event": name, "args": {...}}` normal form.
impl Event {
    /// ABI event name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Registered { .. } => "Registered",
            Self::UriUpdated { .. } => "URIUpdated",
            Self::MetadataSet { .. } => "MetadataSet",
            Self::Transfer { .. } => "Transfer",
            Self::Approval { .. } => "Approval",
            Self::ApprovalForAll { .. } => "ApprovalForAll",
            Self::MetadataUpdate { .. } => "MetadataUpdate",
            Self::BatchMetadataUpdate { .. } => "BatchMetadataUpdate",
            Self::NewFeedback { .. } => "NewFeedback",
            Self::FeedbackRevoked { .. } => "FeedbackRevoked",
            Self::ResponseAppended { .. } => "ResponseAppended",
            Self::ValidationRequest { .. } => "ValidationRequest",
            Self::ValidationResponse { .. } => "ValidationResponse",
        }
    }

    /// Decodes a raw [`Log`]; unknown/admin topics → `Ok(None)`, malformed
    /// payloads under a known topic → [`EventDecodeError::MalformedLog`].
    ///
    /// # Errors
    ///
    /// Returns [`EventDecodeError::MalformedLog`] for truncated topics or data.
    pub fn decode(log: &Log) -> Result<Option<Self>, EventDecodeError> {
        let Some(topic0) = log.topics().first() else {
            return Ok(None);
        };
        if let Some(event) = decode_identity(*topic0, log)? {
            return Ok(Some(event));
        }
        if let Some(event) = decode_reputation(*topic0, log)? {
            return Ok(Some(event));
        }
        decode_validation(*topic0, log)
    }
}

macro_rules! on {
    ($log:expr, $topic0:expr, $ty:ty, $map:expr) => {
        if $topic0 == <$ty>::SIGNATURE_HASH {
            let decoded = <$ty>::decode_log($log).map_err(|e| malformed(format!("{e}")))?;
            #[allow(
                clippy::redundant_closure_call,
                reason = "macro-applied mapper closure"
            )]
            return Ok(Some(($map)(decoded.data)));
        }
    };
}

fn decode_identity(topic0: B256, log: &Log) -> Result<Option<Event>, EventDecodeError> {
    on!(
        log,
        topic0,
        IIdentityRegistry::Registered,
        |e: IIdentityRegistry::Registered| Event::Registered {
            agent_id: e.agentId,
            agent_uri: e.agentURI,
            owner: e.owner,
        }
    );
    on!(
        log,
        topic0,
        IIdentityRegistry::URIUpdated,
        |e: IIdentityRegistry::URIUpdated| Event::UriUpdated {
            agent_id: e.agentId,
            new_uri: e.newURI,
            updated_by: e.updatedBy,
        }
    );
    on!(
        log,
        topic0,
        IIdentityRegistry::MetadataSet,
        |e: IIdentityRegistry::MetadataSet| Event::MetadataSet {
            agent_id: e.agentId,
            indexed_metadata_key: e.indexedMetadataKey,
            metadata_key: e.metadataKey,
            metadata_value: e.metadataValue,
        }
    );
    on!(
        log,
        topic0,
        IIdentityRegistry::Transfer,
        |e: IIdentityRegistry::Transfer| Event::Transfer {
            from: e.from,
            to: e.to,
            token_id: e.tokenId,
        }
    );
    on!(
        log,
        topic0,
        IIdentityRegistry::Approval,
        |e: IIdentityRegistry::Approval| Event::Approval {
            owner: e.owner,
            approved: e.approved,
            token_id: e.tokenId,
        }
    );
    on!(
        log,
        topic0,
        IIdentityRegistry::ApprovalForAll,
        |e: IIdentityRegistry::ApprovalForAll| {
            Event::ApprovalForAll {
                owner: e.owner,
                operator: e.operator,
                approved: e.approved,
            }
        }
    );
    on!(
        log,
        topic0,
        IIdentityRegistry::MetadataUpdate,
        |e: IIdentityRegistry::MetadataUpdate| {
            Event::MetadataUpdate {
                token_id: e._tokenId,
            }
        }
    );
    on!(
        log,
        topic0,
        IIdentityRegistry::BatchMetadataUpdate,
        |e: IIdentityRegistry::BatchMetadataUpdate| {
            Event::BatchMetadataUpdate {
                from_token_id: e._fromTokenId,
                to_token_id: e._toTokenId,
            }
        }
    );
    Ok(None)
}

fn decode_reputation(topic0: B256, log: &Log) -> Result<Option<Event>, EventDecodeError> {
    on!(
        log,
        topic0,
        IReputationRegistry::NewFeedback,
        |e: IReputationRegistry::NewFeedback| {
            Event::NewFeedback {
                agent_id: e.agentId,
                client_address: e.clientAddress,
                feedback_index: e.feedbackIndex,
                value: e.value,
                value_decimals: e.valueDecimals,
                indexed_tag1: e.indexedTag1,
                tag1: e.tag1,
                tag2: e.tag2,
                endpoint: e.endpoint,
                feedback_uri: e.feedbackURI,
                feedback_hash: e.feedbackHash,
            }
        }
    );
    on!(
        log,
        topic0,
        IReputationRegistry::FeedbackRevoked,
        |e: IReputationRegistry::FeedbackRevoked| {
            Event::FeedbackRevoked {
                agent_id: e.agentId,
                client_address: e.clientAddress,
                feedback_index: e.feedbackIndex,
            }
        }
    );
    on!(
        log,
        topic0,
        IReputationRegistry::ResponseAppended,
        |e: IReputationRegistry::ResponseAppended| {
            Event::ResponseAppended {
                agent_id: e.agentId,
                client_address: e.clientAddress,
                feedback_index: e.feedbackIndex,
                responder: e.responder,
                response_uri: e.responseURI,
                response_hash: e.responseHash,
            }
        }
    );
    Ok(None)
}

fn decode_validation(topic0: B256, log: &Log) -> Result<Option<Event>, EventDecodeError> {
    on!(
        log,
        topic0,
        IValidationRegistry::ValidationRequest,
        |e: IValidationRegistry::ValidationRequest| {
            Event::ValidationRequest {
                validator_address: e.validatorAddress,
                agent_id: e.agentId,
                request_uri: e.requestURI,
                request_hash: e.requestHash,
            }
        }
    );
    on!(
        log,
        topic0,
        IValidationRegistry::ValidationResponse,
        |e: IValidationRegistry::ValidationResponse| {
            Event::ValidationResponse {
                validator_address: e.validatorAddress,
                agent_id: e.agentId,
                request_hash: e.requestHash,
                response: e.response,
                response_uri: e.responseURI,
                response_hash: e.responseHash,
                tag: e.tag,
            }
        }
    );
    Ok(None)
}
