//! Shared revert-reason enum for all three registries. Decoding never fails —
//! unrecognised data stays available as [`RevertReason::Unknown`].

use alloy::primitives::{Address, U256};
use alloy::sol_types::SolError;

use crate::abi::IIdentityRegistry;
use alloy::sol_types::{Panic, Revert};

/// One shared reason enum covering `Error(string)` reasons, the Identity ABI
/// custom errors, `Panic(uint256)`, and everything else.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum RevertReason {
    /// `"Not authorized"`
    NotAuthorized,
    /// `"reserved key"`
    ReservedMetadataKey,
    /// `"bad wallet"`
    InvalidWallet,
    /// `"expired"`
    DeadlineExpired,
    /// `"deadline too far"`
    DeadlineTooFar,
    /// `"invalid wallet sig"`
    InvalidWalletSignature,
    /// `"too many decimals"`
    TooManyDecimals,
    /// `"value too large"`
    ValueOutOfRange,
    /// `"Self-feedback not allowed"`
    SelfFeedback,
    /// `"index must be > 0"`
    InvalidFeedbackIndex,
    /// `"index out of bounds"`
    FeedbackIndexOutOfBounds,
    /// `"Already revoked"`
    AlreadyRevoked,
    /// `"Empty URI"`
    EmptyResponseUri,
    /// `"clientAddresses required"`
    ClientAddressesRequired,
    /// `"bad validator"`
    InvalidValidator,
    /// `"exists"`
    RequestExists,
    /// `"unknown"`
    UnknownRequest,
    /// `"not validator"`
    NotValidator,
    /// `"resp>100"`
    ResponseOutOfRange,
    /// Any other `Error(string)` message.
    Message {
        /// The unrecognized `Error(string)` payload.
        message: String,
    },
    /// `ERC721NonexistentToken(tokenId)`.
    AgentNotFound {
        /// The agent id.
        agent_id: U256,
    },
    /// `ERC721IncorrectOwner(sender, tokenId, owner)`.
    IncorrectOwner {
        /// The sender address.
        sender: Address,
        /// The agent id.
        agent_id: U256,
        /// The owner address.
        owner: Address,
    },
    /// `ERC721InsufficientApproval(operator, tokenId)`.
    InsufficientApproval {
        /// The operator address.
        operator: Address,
        /// The agent id.
        agent_id: U256,
    },
    /// `ERC721InvalidReceiver(receiver)`.
    InvalidReceiver {
        /// The receiver address.
        receiver: Address,
    },
    /// `ERC721InvalidSender(sender)`.
    InvalidSender {
        /// The sender address.
        sender: Address,
    },
    /// `ERC721InvalidOwner(owner)`.
    InvalidOwner {
        /// The owner address.
        owner: Address,
    },
    /// `ERC721InvalidApprover(approver)`.
    InvalidApprover {
        /// The approver address.
        approver: Address,
    },
    /// `ERC721InvalidOperator(operator)`.
    InvalidOperator {
        /// The operator address.
        operator: Address,
    },
    /// `Panic(uint256)`.
    Panic {
        /// The panic code.
        code: U256,
    },
    /// Anything else — admin errors, empty data, garbage.
    Unknown {
        /// The raw revert data.
        data: Vec<u8>,
    },
}

impl std::fmt::Display for RevertReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotAuthorized => write!(f, "not authorized"),
            Self::ReservedMetadataKey => write!(f, "reserved metadata key"),
            Self::InvalidWallet => write!(f, "invalid wallet"),
            Self::DeadlineExpired => write!(f, "deadline expired"),
            Self::DeadlineTooFar => write!(f, "deadline too far"),
            Self::InvalidWalletSignature => write!(f, "invalid wallet signature"),
            Self::TooManyDecimals => write!(f, "too many decimals"),
            Self::ValueOutOfRange => write!(f, "value out of range"),
            Self::SelfFeedback => write!(f, "self-feedback not allowed"),
            Self::InvalidFeedbackIndex => write!(f, "feedback index must be > 0"),
            Self::FeedbackIndexOutOfBounds => write!(f, "feedback index out of bounds"),
            Self::AlreadyRevoked => write!(f, "already revoked"),
            Self::EmptyResponseUri => write!(f, "empty response uri"),
            Self::ClientAddressesRequired => write!(f, "client addresses required"),
            Self::InvalidValidator => write!(f, "invalid validator"),
            Self::RequestExists => write!(f, "request already exists"),
            Self::UnknownRequest => write!(f, "unknown request"),
            Self::NotValidator => write!(f, "not validator"),
            Self::ResponseOutOfRange => write!(f, "response out of range"),
            Self::Message { message } => write!(f, "revert: {message}"),
            Self::AgentNotFound { agent_id } => write!(f, "agent {agent_id} not found"),
            Self::IncorrectOwner { sender, agent_id, owner } => write!(
                f,
                "incorrect owner: {sender} does not own agent {agent_id} (owner {owner})"
            ),
            Self::InsufficientApproval { operator, agent_id } => {
                write!(f, "insufficient approval: {operator} for agent {agent_id}")
            }
            Self::InvalidReceiver { receiver } => write!(f, "invalid receiver {receiver}"),
            Self::InvalidSender { sender } => write!(f, "invalid sender {sender}"),
            Self::InvalidOwner { owner } => write!(f, "invalid owner {owner}"),
            Self::InvalidApprover { approver } => write!(f, "invalid approver {approver}"),
            Self::InvalidOperator { operator } => write!(f, "invalid operator {operator}"),
            Self::Panic { code } => write!(f, "panic: {code}"),
            Self::Unknown { data } => write!(f, "unknown revert: 0x{}", hex_encode(data)),
        }
    }
}

fn hex_encode(data: &[u8]) -> String {
    let mut out = String::with_capacity(data.len() * 2);
    for &b in data {
        // both nibbles are < 16 so from_digit always yields a char
        if let (Some(hi), Some(lo)) =
            (char::from_digit(u32::from(b >> 4), 16), char::from_digit(u32::from(b & 0x0f), 16))
        {
            out.push(hi);
            out.push(lo);
        }
    }
    out
}

fn error_string_reason(message: &str) -> RevertReason {
    match message {
        "Not authorized" => RevertReason::NotAuthorized,
        "reserved key" => RevertReason::ReservedMetadataKey,
        "bad wallet" => RevertReason::InvalidWallet,
        "expired" => RevertReason::DeadlineExpired,
        "deadline too far" => RevertReason::DeadlineTooFar,
        "invalid wallet sig" => RevertReason::InvalidWalletSignature,
        "too many decimals" => RevertReason::TooManyDecimals,
        "value too large" => RevertReason::ValueOutOfRange,
        "Self-feedback not allowed" => RevertReason::SelfFeedback,
        "index must be > 0" => RevertReason::InvalidFeedbackIndex,
        "index out of bounds" => RevertReason::FeedbackIndexOutOfBounds,
        "Already revoked" => RevertReason::AlreadyRevoked,
        "Empty URI" => RevertReason::EmptyResponseUri,
        "clientAddresses required" => RevertReason::ClientAddressesRequired,
        "bad validator" => RevertReason::InvalidValidator,
        "exists" => RevertReason::RequestExists,
        "unknown" => RevertReason::UnknownRequest,
        "not validator" => RevertReason::NotValidator,
        "resp>100" => RevertReason::ResponseOutOfRange,
        other => RevertReason::Message {
            message: other.into(),
        },
    }
}

fn unknown(data: &[u8]) -> RevertReason {
    RevertReason::Unknown {
        data: data.to_vec(),
    }
}

fn selector(data: &[u8]) -> Option<[u8; 4]> {
    data.get(..4)?.try_into().ok()
}

impl RevertReason {
    /// Decodes revert data; never fails.
    #[must_use]
    pub fn decode(data: &[u8]) -> Self {
        let Some(sel) = selector(data) else {
            return unknown(data);
        };
        if sel == Revert::SELECTOR {
            return Revert::abi_decode(data)
                .map_or_else(|_| unknown(data), |e| error_string_reason(&e.reason));
        }
        if sel == Panic::SELECTOR {
            return Panic::abi_decode(data)
                .map_or_else(|_| unknown(data), |e| Self::Panic { code: e.code });
        }
        if sel == IIdentityRegistry::ERC721NonexistentToken::SELECTOR {
            return IIdentityRegistry::ERC721NonexistentToken::abi_decode(data).map_or_else(
                |_| unknown(data),
                |e| Self::AgentNotFound {
                    agent_id: e.tokenId,
                },
            );
        }
        if sel == IIdentityRegistry::ERC721IncorrectOwner::SELECTOR {
            return IIdentityRegistry::ERC721IncorrectOwner::abi_decode(data).map_or_else(
                |_| unknown(data),
                |e| Self::IncorrectOwner {
                    sender: e.sender,
                    agent_id: e.tokenId,
                    owner: e.owner,
                },
            );
        }
        if sel == IIdentityRegistry::ERC721InsufficientApproval::SELECTOR {
            return IIdentityRegistry::ERC721InsufficientApproval::abi_decode(data).map_or_else(
                |_| unknown(data),
                |e| Self::InsufficientApproval {
                    operator: e.operator,
                    agent_id: e.tokenId,
                },
            );
        }
        if sel == IIdentityRegistry::ERC721InvalidReceiver::SELECTOR {
            return IIdentityRegistry::ERC721InvalidReceiver::abi_decode(data).map_or_else(
                |_| unknown(data),
                |e| Self::InvalidReceiver {
                    receiver: e.receiver,
                },
            );
        }
        if sel == IIdentityRegistry::ERC721InvalidSender::SELECTOR {
            return IIdentityRegistry::ERC721InvalidSender::abi_decode(data).map_or_else(
                |_| unknown(data),
                |e| Self::InvalidSender { sender: e.sender },
            );
        }
        if sel == IIdentityRegistry::ERC721InvalidOwner::SELECTOR {
            return IIdentityRegistry::ERC721InvalidOwner::abi_decode(data)
                .map_or_else(|_| unknown(data), |e| Self::InvalidOwner { owner: e.owner });
        }
        if sel == IIdentityRegistry::ERC721InvalidApprover::SELECTOR {
            return IIdentityRegistry::ERC721InvalidApprover::abi_decode(data).map_or_else(
                |_| unknown(data),
                |e| Self::InvalidApprover {
                    approver: e.approver,
                },
            );
        }
        if sel == IIdentityRegistry::ERC721InvalidOperator::SELECTOR {
            return IIdentityRegistry::ERC721InvalidOperator::abi_decode(data).map_or_else(
                |_| unknown(data),
                |e| Self::InvalidOperator {
                    operator: e.operator,
                },
            );
        }
        unknown(data)
    }
}
