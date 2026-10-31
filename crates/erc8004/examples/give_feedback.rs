#![allow(
    unused_crate_dependencies,
    clippy::print_stdout,
    reason = "example prints results to stdout"
)]

//! Gives on-chain feedback to an agent, signed by a private key from
//! `PRIVATE_KEY`. `RPC_URL` and `AGENT_ID`/`FEEDBACK_VALUE` are optional.

use alloy::{
    network::EthereumWallet, primitives::U256, providers::ProviderBuilder,
    signers::local::PrivateKeySigner, transports::http::reqwest::Url,
};
use erc8004::{Erc8004, FeedbackValue, NewFeedback, deployments};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let key =
        std::env::var("PRIVATE_KEY").map_err(|_| "set PRIVATE_KEY to a hex-encoded signing key")?;
    let signer: PrivateKeySigner = key.parse()?;
    let wallet = EthereumWallet::from(signer);

    let rpc_url: Url = std::env::var("RPC_URL")
        .unwrap_or_else(|_| "https://mainnet.base.org".into())
        .parse()?;
    let agent_id = std::env::var("AGENT_ID")
        .ok()
        .and_then(|raw| raw.parse::<u64>().ok())
        .unwrap_or(1);
    let value: FeedbackValue = std::env::var("FEEDBACK_VALUE")
        .unwrap_or_else(|_| "87.50".into())
        .parse()?;

    let provider = ProviderBuilder::new().wallet(wallet).connect_http(rpc_url);
    let erc = Erc8004::new(provider, deployments::BASE);

    let feedback = NewFeedback::new(U256::from(agent_id), value)
        .tag1("quality")
        .tag2("example")
        .endpoint("https://agent.example");
    let outcome = erc.reputation().give_feedback(&feedback).await?;
    println!("tx {} — feedback #{}", outcome.tx_hash, outcome.value);
    Ok(())
}
