//! Cross-language anvil end-to-end — mirrors `packages/erc8004/tests/e2e/core.test.ts`.
//! Gated on `ERC8004_E2E_DEPLOYMENT` pointing at `target/e2e/deployment.json`;
//! run via `e2e/run.sh` (`cargo test --all-features --test e2e -- --ignored`).
#![allow(clippy::unwrap_used, reason = "test assertions")]
#![allow(clippy::expect_used, reason = "test assertions")]
#![allow(clippy::indexing_slicing, reason = "fixed-size test fixtures")]
#![allow(clippy::missing_panics_doc, reason = "test only")]
#![allow(
    unused_crate_dependencies,
    reason = "the test target uses only the lib's public API"
)]

use std::str::FromStr;
use std::sync::OnceLock;

use alloy::network::EthereumWallet;
use alloy::primitives::{Address, B256, U256};
use alloy::providers::{Provider, ProviderBuilder};
use alloy::signers::Signer as _;
use alloy::signers::local::PrivateKeySigner;
use erc8004::{
    AgentWalletDeadlineError, AgentWalletSet, Erc8004, FeedbackFilter, FeedbackValue,
    MetadataEntry, NewFeedback, ProviderError, RevertReason, ValidationResponse,
};

// anvil default accounts: #0 owner, #1 client, #2 validator, #3 new wallet
const KEYS: [&str; 4] = [
    "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80",
    "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d",
    "0x5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a",
    "0x7c852118294e51e653712a81e05800f419141751be58f605c371e15141b007a6",
];

struct Env {
    rpc: String,
    chain_id: u64,
    identity: Address,
    reputation: Address,
    validation: Address,
}

fn env() -> &'static Env {
    static ENV: OnceLock<Env> = OnceLock::new();
    ENV.get_or_init(|| {
        let path = std::env::var("ERC8004_E2E_DEPLOYMENT")
            .expect("ERC8004_E2E_DEPLOYMENT points at target/e2e/deployment.json");
        let text = std::fs::read_to_string(&path).expect("read deployment.json");
        let parsed: serde_json::Value = serde_json::from_str(&text).expect("parse deployment.json");
        let addr = |key: &str| {
            parsed[key]
                .as_str()
                .and_then(|s| Address::from_str(s).ok())
                .expect("address")
        };
        Env {
            rpc: std::env::var("ERC8004_E2E_RPC")
                .unwrap_or_else(|_| "http://127.0.0.1:8545".to_owned()),
            chain_id: parsed["chainId"].as_u64().expect("chainId"),
            identity: addr("identityRegistry"),
            reputation: addr("reputationRegistry"),
            validation: addr("validationRegistry"),
        }
    })
}

fn erc(key: usize) -> Erc8004<impl Provider> {
    let env = env();
    let signer = PrivateKeySigner::from_str(KEYS[key]).expect("key");
    let wallet = EthereumWallet::from(signer);
    let url = reqwest::Url::parse(&env.rpc).expect("rpc url");
    let provider = ProviderBuilder::new().wallet(wallet).connect_http(url);
    Erc8004::new(provider, deployment())
}

fn addr(key: usize) -> Address {
    PrivateKeySigner::from_str(KEYS[key])
        .expect("key")
        .address()
}

fn deployment() -> erc8004::Deployment {
    let env = env();
    erc8004::Deployment {
        id: "anvil",
        name: "Anvil",
        chain_id: env.chain_id,
        testnet: true,
        identity_registry: env.identity,
        reputation_registry: env.reputation,
        validation_registry: env.validation,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    #[ignore = "requires anvil + deployed registries (e2e/run.sh)"]
    async fn register_read_update() {
        let erc = erc(0);
        let outcome = erc
            .identity()
            .register_agent(
                "ipfs://bafkrei/rust-agent",
                &[MetadataEntry::new(
                    "endpoint",
                    alloy::primitives::Bytes::from_static(b"hello"),
                )],
            )
            .await
            .expect("register");
        let agent_id = outcome.value;
        assert!(agent_id > U256::ZERO);

        let agent = erc.identity().get_agent(agent_id).await.expect("get_agent");
        assert_eq!(agent.owner, addr(0));
        assert_eq!(agent.agent_uri, "ipfs://bafkrei/rust-agent");
        // register() defaults the agent wallet to the owner
        assert_eq!(agent.agent_wallet, Some(addr(0)));

        erc.identity()
            .set_agent_uri(agent_id, "https://a.test/agent.json")
            .await
            .expect("set_agent_uri");
        assert_eq!(
            erc.identity().get_agent_uri(agent_id).await.unwrap(),
            "https://a.test/agent.json"
        );

        erc.identity()
            .set_metadata(
                agent_id,
                "version",
                alloy::primitives::Bytes::from(vec![1u8]),
            )
            .await
            .expect("set_metadata");
        assert_eq!(
            erc.identity()
                .get_metadata(agent_id, "version")
                .await
                .unwrap(),
            alloy::primitives::Bytes::from(vec![1u8])
        );

        // reserved key → client guard
        let err = erc
            .identity()
            .set_metadata(
                agent_id,
                "agentWallet",
                alloy::primitives::Bytes::from(vec![1u8]),
            )
            .await
            .unwrap_err();
        assert!(matches!(err, ProviderError::ReservedMetadataKey));
    }

    #[tokio::test]
    #[ignore = "requires anvil + deployed registries (e2e/run.sh)"]
    async fn agent_wallet_lifecycle() {
        let erc = erc(0);
        let outcome = erc
            .identity()
            .register_agent("", &[])
            .await
            .expect("register");
        let agent_id = outcome.value;

        let provider = erc.provider();
        let block = provider
            .get_block(alloy::rpc::types::BlockId::latest())
            .await
            .unwrap()
            .unwrap();
        let now = block.header.timestamp;
        let deadline = U256::from(now + 60);

        // sign by the new wallet (account #3) via the shared EIP-712 helpers
        let signer3 = PrivateKeySigner::from_str(KEYS[3]).expect("key");
        let wallet3 = signer3.address();
        let signing = AgentWalletSet {
            agentId: agent_id,
            newWallet: wallet3,
            owner: addr(0),
            deadline,
        };
        let hash = signing.signing_hash(env().chain_id, env().identity);
        let signature = signer3.sign_hash(&hash).await.expect("sign");
        let sig_bytes: alloy::primitives::Bytes = signature.as_bytes().into();

        erc.identity()
            .set_agent_wallet(agent_id, wallet3, deadline, sig_bytes)
            .await
            .expect("set_agent_wallet");
        assert_eq!(
            erc.identity().get_agent_wallet(agent_id).await.unwrap(),
            Some(wallet3)
        );

        // too-far deadline → client guard
        let far = U256::from(now + 600);
        let far_signing = AgentWalletSet {
            agentId: agent_id,
            newWallet: wallet3,
            owner: addr(0),
            deadline: far,
        };
        let far_hash = far_signing.signing_hash(env().chain_id, env().identity);
        let far_sig = signer3.sign_hash(&far_hash).await.expect("sign");
        let far_bytes: alloy::primitives::Bytes = far_sig.as_bytes().into();
        let err = erc
            .identity()
            .set_agent_wallet(agent_id, wallet3, far, far_bytes)
            .await
            .unwrap_err();
        assert!(matches!(
            err,
            ProviderError::Deadline(AgentWalletDeadlineError::DeadlineTooFar { .. })
        ));

        erc.identity()
            .unset_agent_wallet(agent_id)
            .await
            .expect("unset");
        assert_eq!(
            erc.identity().get_agent_wallet(agent_id).await.unwrap(),
            None
        );
    }

    #[tokio::test]
    #[ignore = "requires anvil + deployed registries (e2e/run.sh)"]
    async fn feedback_lifecycle() {
        let erc0 = erc(0);
        let erc1 = erc(1);
        let outcome = erc0
            .identity()
            .register_agent("", &[])
            .await
            .expect("register");
        let agent_id = outcome.value;

        let value: FeedbackValue = "87.50".parse().expect("feedback value");
        let fb = erc1
            .reputation()
            .give_feedback(
                &NewFeedback::new(agent_id, value)
                    .tag1("quality")
                    .tag2("speed"),
            )
            .await
            .expect("give_feedback");
        assert_eq!(fb.value, 1);

        // self-feedback → revert
        let err = erc0
            .reputation()
            .give_feedback(&NewFeedback::new(agent_id, value))
            .await
            .unwrap_err();
        assert!(matches!(
            err,
            ProviderError::Revert(RevertReason::SelfFeedback)
        ));

        let read = erc1
            .reputation()
            .read_feedback(agent_id, addr(1), fb.value)
            .await
            .expect("read_feedback");
        assert_eq!(read.value.value(), 8750);
        assert_eq!(read.value.decimals(), 2);
        assert_eq!(read.tag1, "quality");
        assert!(!read.is_revoked);

        let all = erc1
            .reputation()
            .read_all_feedback(agent_id, &FeedbackFilter::new(vec![addr(1)]), false)
            .await
            .expect("read_all_feedback");
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].feedback_index, fb.value);

        let summary = erc1
            .reputation()
            .get_reputation_summary(agent_id, &FeedbackFilter::new(vec![addr(1)]))
            .await
            .expect("summary");
        assert_eq!(summary.count, 1);
        assert_eq!(summary.value.value(), 8750);

        // empty clients → guard
        let err2 = erc1
            .reputation()
            .get_reputation_summary(agent_id, &FeedbackFilter::default())
            .await
            .unwrap_err();
        assert!(matches!(err2, ProviderError::ClientAddressesRequired));

        erc1.reputation()
            .append_response(agent_id, addr(1), fb.value, "ipfs://response", B256::ZERO)
            .await
            .expect("append_response");
        let count = erc1
            .reputation()
            .get_response_count(agent_id, Some(addr(1)), Some(fb.value), &[addr(1)])
            .await
            .expect("get_response_count");
        assert_eq!(count, 1);

        erc1.reputation()
            .revoke_feedback(agent_id, fb.value)
            .await
            .expect("revoke");
        let err3 = erc1
            .reputation()
            .revoke_feedback(agent_id, fb.value)
            .await
            .unwrap_err();
        assert!(matches!(
            err3,
            ProviderError::Revert(RevertReason::AlreadyRevoked)
        ));

        // empty response URI → guard
        let err4 = erc1
            .reputation()
            .append_response(agent_id, addr(1), fb.value, "", B256::ZERO)
            .await
            .unwrap_err();
        assert!(matches!(err4, ProviderError::EmptyResponseUri));
    }

    #[tokio::test]
    #[ignore = "requires anvil + deployed registries (e2e/run.sh)"]
    async fn validation_lifecycle() {
        let erc0 = erc(0);
        let erc1 = erc(1);
        let erc2 = erc(2);
        let outcome = erc0
            .identity()
            .register_agent("", &[])
            .await
            .expect("register");
        let agent_id = outcome.value;
        let request_hash = alloy::primitives::keccak256(format!("req-{agent_id}-rust").as_bytes());

        erc0.validation()
            .request_validation(addr(2), agent_id, "ipfs://req", request_hash)
            .await
            .expect("request_validation");

        // out-of-range → guard (101 > 100)
        let err = erc2
            .validation()
            .respond_to_validation(&ValidationResponse::new(request_hash, 101))
            .await
            .unwrap_err();
        assert!(matches!(err, ProviderError::ResponseOutOfRange));

        // wrong responder → revert NotValidator
        let err2 = erc1
            .validation()
            .respond_to_validation(&ValidationResponse::new(request_hash, 50))
            .await
            .unwrap_err();
        assert!(matches!(
            err2,
            ProviderError::Revert(RevertReason::NotValidator)
        ));

        erc2.validation()
            .respond_to_validation(&ValidationResponse::new(request_hash, 100).tag("hard"))
            .await
            .expect("respond_to_validation");
        let status = erc0
            .validation()
            .get_validation_status(request_hash)
            .await
            .expect("status")
            .expect("present");
        assert_eq!(status.validator, addr(2));
        assert_eq!(status.response, 100);
        assert_eq!(status.tag, "hard");

        let summary = erc0
            .validation()
            .get_validation_summary(agent_id, &[addr(2)], "")
            .await
            .expect("summary");
        assert_eq!(summary.count, 1);
        assert_eq!(summary.average_response, 100);

        // unknown request → None
        let none = erc0
            .validation()
            .get_validation_status(alloy::primitives::keccak256(b"nope"))
            .await
            .expect("status");
        assert_eq!(none, None);
    }

    #[tokio::test]
    #[ignore = "requires anvil + deployed registries (e2e/run.sh)"]
    async fn transfer_clears_wallet() {
        let erc0 = erc(0);
        let outcome = erc0
            .identity()
            .register_agent("", &[])
            .await
            .expect("register");
        let agent_id = outcome.value;

        let provider = erc0.provider();
        let block = provider
            .get_block(alloy::rpc::types::BlockId::latest())
            .await
            .unwrap()
            .unwrap();
        let now = block.header.timestamp;

        let signer3 = PrivateKeySigner::from_str(KEYS[3]).expect("key");
        let wallet3 = signer3.address();
        let deadline = U256::from(now + 60);
        let signing = AgentWalletSet {
            agentId: agent_id,
            newWallet: wallet3,
            owner: addr(0),
            deadline,
        };
        let hash = signing.signing_hash(env().chain_id, env().identity);
        let signature = signer3.sign_hash(&hash).await.expect("sign");
        let sig_bytes: alloy::primitives::Bytes = signature.as_bytes().into();
        erc0.identity()
            .set_agent_wallet(agent_id, wallet3, deadline, sig_bytes)
            .await
            .expect("set_agent_wallet");

        erc0.identity()
            .transfer_agent(addr(1), agent_id)
            .await
            .expect("transfer_agent");
        let agent = erc0
            .identity()
            .get_agent(agent_id)
            .await
            .expect("get_agent");
        assert_eq!(agent.owner, addr(1));
        assert_eq!(agent.agent_wallet, None);

        // nonexistent agent → AgentNotFound revert
        let err = erc0
            .identity()
            .get_agent_owner(U256::from(999_999u64))
            .await
            .unwrap_err();
        assert!(matches!(
            err,
            ProviderError::Revert(RevertReason::AgentNotFound { .. })
        ));
    }
}
