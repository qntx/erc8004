//! Shared-vector runner: every case under `../../vectors/` is executed
//! against the Rust core, matching the TypeScript runner byte for byte.

#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::missing_panics_doc,
    clippy::indexing_slicing,
    clippy::cast_possible_truncation,
    clippy::cast_lossless,
    clippy::cast_sign_loss,
    clippy::missing_errors_doc,
    clippy::uninlined_format_args,
    clippy::manual_string_new,
    clippy::items_after_statements,
    clippy::single_call_fn,
    clippy::too_many_lines,
    clippy::non_ascii_literal,
    clippy::redundant_clone,
    clippy::many_single_char_names,
    clippy::match_same_arms,
    clippy::option_if_let_else,
    clippy::map_unwrap_or,
    clippy::shadow_unrelated,
    clippy::tests_outside_test_module,
    missing_docs,
    unused_crate_dependencies,
    reason = "test code panics loudly on failure and mirrors the vector normal forms"
)]
use base64 as _;
use thiserror as _;

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use alloy::primitives::{Address, Bytes, U256};
use erc8004::*;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn load(path: &str) -> Vec<serde_json::Value> {
    let text = fs::read_to_string(root().join(path)).expect(path);
    let doc: serde_json::Value = serde_json::from_str(&text).expect(path);
    doc["cases"].as_array().expect("cases").clone()
}

fn name_of(case: &serde_json::Value) -> String {
    if let Some(n) = case.get("name").and_then(|n| n.as_str()) {
        return n.to_owned();
    }
    let input = case.get("input").or_else(|| case.get("inputText"));
    let title = input
        .map(|i| serde_json::to_string(i).unwrap_or_default())
        .unwrap_or_default();
    let mut title = title;
    if title.len() > 80 {
        title.truncate(80);
        title.push('…');
    }
    title
}

fn hex_to_bytes(s: &str) -> Vec<u8> {
    let s = s.strip_prefix("0x").unwrap_or(s);
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).expect("hex"))
        .collect()
}

fn bytes_hex(b: &[u8]) -> String {
    format!("0x{}", alloy::hex::encode(b))
}

fn address(s: &str) -> Address {
    s.parse::<Address>()
        .or_else(|_| Address::parse_checksummed(s, None))
        .expect("address")
}

fn expect_error_code(err_code: &str, expected: &serde_json::Value, ctx: &str) {
    assert_eq!(err_code, expected.as_str().unwrap_or(""), "code for {ctx}");
}

fn expect_field(err: &JsonDocumentError, case: &serde_json::Value, ctx: &str) {
    if let Some(f) = case.get("field") {
        assert_eq!(
            err.field().unwrap_or(""),
            f.as_str().unwrap_or(""),
            "field for {ctx}"
        );
    }
}

fn revert_to_json(reason: &RevertReason) -> serde_json::Value {
    use serde_json::{Map, Value};
    let mut o = Map::new();
    match reason {
        RevertReason::NotAuthorized => o.insert("reason".into(), "NotAuthorized".into()),
        RevertReason::ReservedMetadataKey => {
            o.insert("reason".into(), "ReservedMetadataKey".into())
        }
        RevertReason::InvalidWallet => o.insert("reason".into(), "InvalidWallet".into()),
        RevertReason::DeadlineExpired => o.insert("reason".into(), "DeadlineExpired".into()),
        RevertReason::DeadlineTooFar => o.insert("reason".into(), "DeadlineTooFar".into()),
        RevertReason::InvalidWalletSignature => {
            o.insert("reason".into(), "InvalidWalletSignature".into())
        }
        RevertReason::TooManyDecimals => o.insert("reason".into(), "TooManyDecimals".into()),
        RevertReason::ValueOutOfRange => o.insert("reason".into(), "ValueOutOfRange".into()),
        RevertReason::SelfFeedback => o.insert("reason".into(), "SelfFeedback".into()),
        RevertReason::InvalidFeedbackIndex => {
            o.insert("reason".into(), "InvalidFeedbackIndex".into())
        }
        RevertReason::FeedbackIndexOutOfBounds => {
            o.insert("reason".into(), "FeedbackIndexOutOfBounds".into())
        }
        RevertReason::AlreadyRevoked => o.insert("reason".into(), "AlreadyRevoked".into()),
        RevertReason::EmptyResponseUri => o.insert("reason".into(), "EmptyResponseUri".into()),
        RevertReason::ClientAddressesRequired => {
            o.insert("reason".into(), "ClientAddressesRequired".into())
        }
        RevertReason::InvalidValidator => o.insert("reason".into(), "InvalidValidator".into()),
        RevertReason::RequestExists => o.insert("reason".into(), "RequestExists".into()),
        RevertReason::UnknownRequest => o.insert("reason".into(), "UnknownRequest".into()),
        RevertReason::NotValidator => o.insert("reason".into(), "NotValidator".into()),
        RevertReason::ResponseOutOfRange => o.insert("reason".into(), "ResponseOutOfRange".into()),
        RevertReason::Message { message } => {
            o.insert("reason".into(), "Message".into());
            o.insert("message".into(), Value::from(message.clone()))
        }
        RevertReason::AgentNotFound { agent_id } => {
            o.insert("reason".into(), "AgentNotFound".into());
            o.insert("agentId".into(), Value::from(agent_id.to_string()))
        }
        RevertReason::IncorrectOwner {
            sender,
            agent_id,
            owner,
        } => {
            o.insert("reason".into(), "IncorrectOwner".into());
            o.insert("sender".into(), Value::from(sender.to_checksum(None)));
            o.insert("agentId".into(), Value::from(agent_id.to_string()));
            o.insert("owner".into(), Value::from(owner.to_checksum(None)))
        }
        RevertReason::InsufficientApproval { operator, agent_id } => {
            o.insert("reason".into(), "InsufficientApproval".into());
            o.insert("operator".into(), Value::from(operator.to_checksum(None)));
            o.insert("agentId".into(), Value::from(agent_id.to_string()))
        }
        RevertReason::InvalidReceiver { receiver } => {
            o.insert("reason".into(), "InvalidReceiver".into());
            o.insert("receiver".into(), Value::from(receiver.to_checksum(None)))
        }
        RevertReason::InvalidSender { sender } => {
            o.insert("reason".into(), "InvalidSender".into());
            o.insert("sender".into(), Value::from(sender.to_checksum(None)))
        }
        RevertReason::InvalidOwner { owner } => {
            o.insert("reason".into(), "InvalidOwner".into());
            o.insert("owner".into(), Value::from(owner.to_checksum(None)))
        }
        RevertReason::InvalidApprover { approver } => {
            o.insert("reason".into(), "InvalidApprover".into());
            o.insert("approver".into(), Value::from(approver.to_checksum(None)))
        }
        RevertReason::InvalidOperator { operator } => {
            o.insert("reason".into(), "InvalidOperator".into());
            o.insert("operator".into(), Value::from(operator.to_checksum(None)))
        }
        RevertReason::Panic { code } => {
            o.insert("reason".into(), "Panic".into());
            o.insert("code".into(), Value::from(code.to_string()))
        }
        RevertReason::Unknown { data } => {
            o.insert("reason".into(), "Unknown".into());
            o.insert("data".into(), Value::from(bytes_hex(data)))
        }
        _ => o.insert("reason".into(), "__non_exhaustive__".into()),
    };
    Value::Object(o)
}

fn event_to_json(event: &Event) -> serde_json::Value {
    use serde_json::{Map, Value};
    let mut args = Map::new();
    let u = |v: &U256| Value::from(v.to_string());
    let a = |v: &Address| Value::from(v.to_checksum(None));
    let h = |v: &alloy::primitives::B256| Value::from(format!("{v}"));
    let b = |v: &Bytes| Value::from(format!("{v}"));
    let d = |v: &dyn std::fmt::Display| Value::from(v.to_string());
    match event {
        Event::Registered {
            agent_id,
            agent_uri,
            owner,
        } => {
            args.insert("agentId".into(), u(agent_id));
            args.insert("owner".into(), a(owner));
            args.insert("agentURI".into(), Value::from(agent_uri.clone()));
        }
        Event::UriUpdated {
            agent_id,
            new_uri,
            updated_by,
        } => {
            args.insert("agentId".into(), u(agent_id));
            args.insert("updatedBy".into(), a(updated_by));
            args.insert("newURI".into(), Value::from(new_uri.clone()));
        }
        Event::MetadataSet {
            agent_id,
            indexed_metadata_key,
            metadata_key,
            metadata_value,
        } => {
            args.insert("agentId".into(), u(agent_id));
            args.insert("indexedMetadataKey".into(), h(indexed_metadata_key));
            args.insert("metadataKey".into(), Value::from(metadata_key.clone()));
            args.insert("metadataValue".into(), b(metadata_value));
        }
        Event::Transfer { from, to, token_id } => {
            args.insert("from".into(), a(from));
            args.insert("to".into(), a(to));
            args.insert("tokenId".into(), u(token_id));
        }
        Event::Approval {
            owner,
            approved,
            token_id,
        } => {
            args.insert("owner".into(), a(owner));
            args.insert("approved".into(), a(approved));
            args.insert("tokenId".into(), u(token_id));
        }
        Event::ApprovalForAll {
            owner,
            operator,
            approved,
        } => {
            args.insert("owner".into(), a(owner));
            args.insert("operator".into(), a(operator));
            args.insert("approved".into(), Value::from(*approved));
        }
        Event::MetadataUpdate { token_id } => {
            args.insert("_tokenId".into(), u(token_id));
        }
        Event::BatchMetadataUpdate {
            from_token_id,
            to_token_id,
        } => {
            args.insert("_fromTokenId".into(), u(from_token_id));
            args.insert("_toTokenId".into(), u(to_token_id));
        }
        Event::NewFeedback {
            agent_id,
            client_address,
            feedback_index,
            value,
            value_decimals,
            indexed_tag1,
            tag1,
            tag2,
            endpoint,
            feedback_uri,
            feedback_hash,
        } => {
            args.insert("agentId".into(), u(agent_id));
            args.insert("clientAddress".into(), a(client_address));
            args.insert("feedbackIndex".into(), d(feedback_index));
            args.insert("value".into(), d(value));
            args.insert("valueDecimals".into(), d(value_decimals));
            args.insert("indexedTag1".into(), h(indexed_tag1));
            args.insert("tag1".into(), Value::from(tag1.clone()));
            args.insert("tag2".into(), Value::from(tag2.clone()));
            args.insert("endpoint".into(), Value::from(endpoint.clone()));
            args.insert("feedbackURI".into(), Value::from(feedback_uri.clone()));
            args.insert("feedbackHash".into(), h(feedback_hash));
        }
        Event::FeedbackRevoked {
            agent_id,
            client_address,
            feedback_index,
        } => {
            args.insert("agentId".into(), u(agent_id));
            args.insert("clientAddress".into(), a(client_address));
            args.insert("feedbackIndex".into(), d(feedback_index));
        }
        Event::ResponseAppended {
            agent_id,
            client_address,
            feedback_index,
            responder,
            response_uri,
            response_hash,
        } => {
            args.insert("agentId".into(), u(agent_id));
            args.insert("clientAddress".into(), a(client_address));
            args.insert("feedbackIndex".into(), d(feedback_index));
            args.insert("responder".into(), a(responder));
            args.insert("responseURI".into(), Value::from(response_uri.clone()));
            args.insert("responseHash".into(), h(response_hash));
        }
        Event::ValidationRequest {
            validator_address,
            agent_id,
            request_uri,
            request_hash,
        } => {
            args.insert("validatorAddress".into(), a(validator_address));
            args.insert("agentId".into(), u(agent_id));
            args.insert("requestURI".into(), Value::from(request_uri.clone()));
            args.insert("requestHash".into(), h(request_hash));
        }
        Event::ValidationResponse {
            validator_address,
            agent_id,
            request_hash,
            response,
            response_uri,
            response_hash,
            tag,
        } => {
            args.insert("validatorAddress".into(), a(validator_address));
            args.insert("agentId".into(), u(agent_id));
            args.insert("requestHash".into(), h(request_hash));
            args.insert("response".into(), d(response));
            args.insert("responseURI".into(), Value::from(response_uri.clone()));
            args.insert("responseHash".into(), h(response_hash));
            args.insert("tag".into(), Value::from(tag.clone()));
        }
        _ => panic!("non-exhaustive event"),
    }
    serde_json::json!({ "event": event.name(), "args": args })
}

#[test]
fn core_agent_registry() {
    for case in load("vectors/core/agent-registry.json") {
        let input = case["input"].as_str().unwrap();
        let ctx = name_of(&case);
        match AgentRegistry::parse(input) {
            Ok(r) => {
                assert!(case.get("error").is_none(), "unexpected ok for {ctx}");
                assert_eq!(r.to_string(), case["output"].as_str().unwrap(), "{ctx}");
                assert_eq!(
                    r.chain_id.to_string(),
                    input.split(':').nth(1).unwrap(),
                    "chain {ctx}"
                );
            }
            Err(e) => {
                let expected = case
                    .get("error")
                    .unwrap_or_else(|| panic!("unexpected err {e} for {ctx}"));
                expect_error_code(e.code(), expected, &ctx);
            }
        }
    }
}

#[test]
fn core_feedback_value() {
    for case in load("vectors/core/feedback-value.json") {
        let ctx = name_of(&case);
        let op = case["op"].as_str().unwrap_or("parse");
        match op {
            "parse" => match case["input"].as_str().unwrap().parse::<FeedbackValue>() {
                Ok(v) => {
                    let out = &case["output"];
                    assert_eq!(
                        v.value().to_string(),
                        out["value"].as_str().unwrap(),
                        "{ctx}"
                    );
                    assert_eq!(
                        v.decimals().to_string(),
                        out["decimals"].as_str().unwrap(),
                        "{ctx}"
                    );
                }
                Err(e) => {
                    expect_error_code(e.code(), &case["error"], &ctx);
                }
            },
            "format" => {
                let i = &case["input"];
                let value: i128 = i["value"].as_str().unwrap().parse().unwrap();
                let decimals: u8 = i["decimals"].as_str().unwrap().parse().unwrap();
                match FeedbackValue::new(value, decimals) {
                    Ok(v) => assert_eq!(v.to_string(), case["output"].as_str().unwrap(), "{ctx}"),
                    Err(e) => expect_error_code(e.code(), &case["error"], &ctx),
                }
            }
            "new" => {
                let i = &case["input"];
                let value: i128 = i["value"].as_str().unwrap().parse().unwrap();
                let decimals: u8 = i["decimals"].as_str().unwrap().parse().unwrap();
                match FeedbackValue::new(value, decimals) {
                    Ok(v) => {
                        let out = &case["output"];
                        assert_eq!(
                            v.value().to_string(),
                            out["value"].as_str().unwrap(),
                            "{ctx}"
                        );
                    }
                    Err(e) => expect_error_code(e.code(), &case["error"], &ctx),
                }
            }
            other => panic!("unknown op {other}"),
        }
    }
}

#[test]
fn core_revert() {
    for case in load("vectors/core/revert.json") {
        let data = hex_to_bytes(case["input"]["data"].as_str().unwrap());
        let reason = RevertReason::decode(&data);
        assert_eq!(
            revert_to_json(&reason),
            case["output"],
            "{}",
            name_of(&case)
        );
    }
}

#[test]
fn registration_data_uri() {
    for case in load("vectors/registration/data-uri.json") {
        let ctx = name_of(&case);
        match case["op"].as_str().unwrap_or("decode") {
            "decode" => match decode_data_uri(case["input"].as_str().unwrap()) {
                Ok(bytes) => {
                    assert_eq!(bytes_hex(&bytes), case["output"].as_str().unwrap(), "{ctx}");
                }
                Err(e) => expect_error_code(e.code(), &case["error"], &ctx),
            },
            "encode" => {
                let bytes = hex_to_bytes(case["input"].as_str().unwrap());
                assert_eq!(
                    encode_data_uri(&bytes),
                    case["output"].as_str().unwrap(),
                    "{ctx}"
                );
            }
            other => panic!("unknown op {other}"),
        }
    }
}

#[test]
fn registration_agent_uri() {
    for case in load("vectors/registration/agent-uri.json") {
        let ctx = name_of(&case);
        let uri = case["input"].as_str().unwrap();
        let result = match case.get("gateway").and_then(|g| g.as_str()) {
            Some(gw) => AgentUri::classify(uri, gw),
            None => AgentUri::classify(uri, DEFAULT_IPFS_GATEWAY),
        };
        match result {
            Ok(classified) => {
                let out = &case["output"];
                let kind = out["kind"].as_str().unwrap();
                match (kind, &classified) {
                    ("NotSet", AgentUri::NotSet) => {}
                    ("Data", AgentUri::Data { bytes }) => {
                        assert_eq!(bytes_hex(bytes), out["bytes"].as_str().unwrap(), "{ctx}");
                    }
                    ("Ipfs", AgentUri::Ipfs { url }) => {
                        assert_eq!(url, out["url"].as_str().unwrap(), "{ctx}");
                    }
                    ("Https", AgentUri::Https { url }) => {
                        assert_eq!(url, out["url"].as_str().unwrap(), "{ctx}");
                    }
                    ("UnsupportedScheme", AgentUri::UnsupportedScheme) => {}
                    (k, v) => panic!("kind mismatch for {ctx}: expected {k}, got {v:?}"),
                }
            }
            Err(e) => expect_error_code(e.code(), &case["error"], &ctx),
        }
    }
}

#[test]
fn registration_file() {
    for case in load("vectors/registration/file.json") {
        let ctx = name_of(&case);
        let result = if let Some(text) = case.get("inputText") {
            RegistrationFile::from_slice(text.as_str().unwrap().as_bytes())
        } else {
            RegistrationFile::from_value(&case["input"])
        };
        match result {
            Ok(file) => assert_eq!(file.to_value(), case["output"], "{ctx}"),
            Err(e) => {
                expect_error_code(e.code(), &case["error"], &ctx);
                expect_field(&e, &case, &ctx);
            }
        }
    }
}

#[test]
fn registration_conformance() {
    for case in load("vectors/registration/conformance.json") {
        let ctx = name_of(&case);
        let file = RegistrationFile::from_value(&case["input"]).expect("parse");
        let issues = file.conformance();
        let expected = case["output"].as_array().unwrap();
        assert_eq!(issues.len(), expected.len(), "issue count for {ctx}");
        for (issue, want) in issues.iter().zip(expected.iter()) {
            assert_eq!(issue.code(), want["code"].as_str().unwrap(), "{ctx}");
            if let Some(index) = want.get("index") {
                let have = match issue {
                    ConformanceIssue::InvalidAgentRegistry { index } => *index,
                    _ => panic!("no index on {issue:?} for {ctx}"),
                };
                assert_eq!(have.to_string(), index.as_str().unwrap(), "index for {ctx}");
            }
        }
    }
}

#[test]
fn registration_domain_verification() {
    for case in load("vectors/registration/domain-verification.json") {
        let ctx = name_of(&case);
        match case["op"].as_str().unwrap_or("url") {
            "url" => match case.get("output") {
                Some(out) if !out.is_null() => {
                    let url = well_known_registration_url(case["input"].as_str().unwrap());
                    assert_eq!(url.as_deref(), Some(out.as_str().unwrap()), "{ctx}");
                }
                _ => {
                    assert!(
                        well_known_registration_url(case["input"].as_str().unwrap()).is_none(),
                        "expected None for {ctx}"
                    );
                }
            },
            "verify" => {
                let file = RegistrationFile::from_value(&case["file"]).expect("parse");
                let registry =
                    AgentRegistry::parse(case["agentRegistry"].as_str().unwrap()).unwrap();
                let agent_id = U256::from_str_radix(case["agentId"].as_str().unwrap(), 10).unwrap();
                assert_eq!(
                    file.has_registration(&registry, agent_id),
                    case["output"].as_bool().unwrap(),
                    "{ctx}"
                );
            }
            other => panic!("unknown op {other}"),
        }
    }
}

#[test]
fn reputation_feedback_file() {
    for case in load("vectors/reputation/feedback-file.json") {
        let ctx = name_of(&case);
        let result = if let Some(text) = case.get("inputText") {
            FeedbackFile::from_slice(text.as_str().unwrap().as_bytes())
        } else {
            FeedbackFile::from_value(&case["input"])
        };
        match result {
            Ok(file) => assert_eq!(file.to_value(), case["output"], "{ctx}"),
            Err(e) => {
                expect_error_code(e.code(), &case["error"], &ctx);
                expect_field(&e, &case, &ctx);
            }
        }
    }
}

#[test]
fn identity_agent_wallet() {
    for case in load("vectors/identity/agent-wallet.json") {
        let ctx = name_of(&case);
        match case["op"].as_str().unwrap_or("hash") {
            "hash" => {
                let i = &case["input"];

                let message = AgentWalletSet {
                    agentId: U256::from_str_radix(i["message"]["agentId"].as_str().unwrap(), 10)
                        .unwrap(),
                    newWallet: address(i["message"]["newWallet"].as_str().unwrap()),
                    owner: address(i["message"]["owner"].as_str().unwrap()),
                    deadline: U256::from_str_radix(i["message"]["deadline"].as_str().unwrap(), 10)
                        .unwrap(),
                };
                let hash = message.signing_hash(
                    i["chainId"].as_u64().unwrap(),
                    address(i["identityRegistry"].as_str().unwrap()),
                );
                assert_eq!(format!("{hash}"), case["output"].as_str().unwrap(), "{ctx}");
            }
            "deadline" => {
                let i = &case["input"];
                let deadline: u64 = i["deadline"].as_str().unwrap().parse().unwrap();
                let now: u64 = i["now"].as_str().unwrap().parse().unwrap();
                match check_agent_wallet_deadline(deadline, now) {
                    Ok(()) => assert!(case["output"].is_null(), "expected ok for {ctx}"),
                    Err(e) => expect_error_code(e.code(), &case["error"], &ctx),
                }
            }
            other => panic!("unknown op {other}"),
        }
    }
}

#[test]
fn events_decode() {
    for case in load("vectors/events/decode.json") {
        let ctx = name_of(&case);
        let topics: Vec<alloy::primitives::B256> = case["input"]["topics"]
            .as_array()
            .unwrap()
            .iter()
            .map(|t| {
                t.as_str()
                    .unwrap()
                    .parse::<alloy::primitives::B256>()
                    .unwrap()
            })
            .collect();
        let data = Bytes::from(hex_to_bytes(case["input"]["data"].as_str().unwrap()));
        let log = alloy::primitives::Log::new_unchecked(Address::ZERO, topics, data);
        match Event::decode(&log) {
            Ok(Some(event)) => {
                assert_eq!(event_to_json(&event), case["output"], "{ctx}");
            }
            Ok(None) => assert!(case["output"].is_null(), "expected None for {ctx}"),
            Err(e) => {
                let mut code = e.code();
                if code == "MalformedLog" {
                    code = "MalformedLog";
                }
                expect_error_code(code, &case["error"], &ctx);
            }
        }
    }
}

#[test]
fn deployments_table() {
    let text = fs::read_to_string(root().join("data/deployments.json")).expect("deployments");
    let doc: serde_json::Value = serde_json::from_str(&text).unwrap();
    let table: BTreeMap<&str, &serde_json::Value> = doc["deployments"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| (d["id"].as_str().unwrap(), d))
        .collect();
    for deployment in deployments::ALL {
        let want = table
            .iter()
            .find(|(id, _)| **id == deployment.id)
            .map(|(_, v)| *v)
            .unwrap_or_else(|| panic!("{} missing from data/deployments.json", deployment.id));
        assert_eq!(deployment.name, want["name"].as_str().unwrap());
        assert_eq!(deployment.chain_id, want["chainId"].as_u64().unwrap());
        assert_eq!(deployment.testnet, want["testnet"].as_bool().unwrap());
        assert_eq!(
            deployment.identity_registry.to_checksum(None),
            want["identityRegistry"].as_str().unwrap()
        );
        assert_eq!(
            deployment.reputation_registry.to_checksum(None),
            want["reputationRegistry"].as_str().unwrap()
        );
        assert_eq!(
            deployment.validation_registry.to_checksum(None),
            want["validationRegistry"].as_str().unwrap()
        );
    }
    assert_eq!(deployments::ALL.len(), table.len(), "count");
    for deployment in deployments::ALL {
        assert!(by_chain_id(deployment.chain_id).is_some_and(|d| d == deployment));
    }
    assert!(by_chain_id(0).is_none());
}

#[test]
fn multibyte_inputs_do_not_panic() {
    // Non-ASCII prefixes must not panic on char-boundary slicing.
    assert!(matches!(
        AgentUri::classify("dαta:x", DEFAULT_IPFS_GATEWAY),
        Ok(AgentUri::UnsupportedScheme)
    ));
    assert!(decode_data_uri("dαta:application/json;base64,e30=").is_err());
    assert!(well_known_registration_url("https://exämple.com").is_some());
    assert!("β".parse::<FeedbackValue>().is_err());
    assert!("-β".parse::<FeedbackValue>().is_err());
}
