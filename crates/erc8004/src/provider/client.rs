//! `eth_call`/`send_transaction` internals — revert decoding lives here.

use alloy::primitives::Address;
use alloy::providers::Provider;
use alloy::rpc::types::{TransactionReceipt, TransactionRequest};
use alloy::sol_types::SolCall;

use crate::revert::RevertReason;

use super::error::ProviderError;

/// `eth_call`s `to` with `call` and decodes the return. Revert payloads decode to
/// [`ProviderError::Revert`].
///
/// # Errors
///
/// [`ProviderError::Revert`] for contract reverts, [`ProviderError::Rpc`] for transport failures.
pub(crate) async fn call<P: Provider, C: SolCall>(
    provider: &P,
    to: Address,
    call: C,
) -> Result<C::Return, ProviderError> {
    let request = TransactionRequest::default()
        .to(to)
        .input(call.abi_encode().into());
    let result = provider.call(request).await.map_err(map_rpc_error)?;
    C::abi_decode_returns(&result).map_err(|e| ProviderError::InvalidResponse(format!("{e}")))
}

/// Sends `to` with `call`, waits for the receipt, and requires `status = true`.
///
/// # Errors
///
/// [`ProviderError::Revert`] for pre-send reverts, [`ProviderError::TransactionFailed`] for a
/// failed receipt, [`ProviderError::Rpc`] for transport failures.
pub(crate) async fn send<P: Provider, C: SolCall>(
    provider: &P,
    to: Address,
    call: C,
) -> Result<TransactionReceipt, ProviderError> {
    let request = TransactionRequest::default()
        .to(to)
        .input(call.abi_encode().into());
    let pending = provider
        .send_transaction(request)
        .await
        .map_err(map_rpc_error)?;
    let receipt = pending
        .get_receipt()
        .await
        .map_err(|e| ProviderError::Rpc(Box::new(e)))?;
    if !receipt.status() {
        return Err(ProviderError::TransactionFailed {
            tx_hash: receipt.transaction_hash,
        });
    }
    Ok(receipt)
}

/// The latest block's timestamp (for `setAgentWallet` deadline checks).
///
/// # Errors
///
/// [`ProviderError::Rpc`] on transport failure; [`ProviderError::InvalidResponse`] when the node
/// has no latest block.
pub(crate) async fn latest_timestamp<P: Provider>(provider: &P) -> Result<u64, ProviderError> {
    let block = provider
        .get_block(alloy::rpc::types::BlockId::latest())
        .await
        .map_err(map_rpc_error)?
        .ok_or_else(|| ProviderError::InvalidResponse("no latest block".into()))?;
    Ok(block.header.timestamp)
}

fn map_rpc_error(error: alloy::transports::TransportError) -> ProviderError {
    if let Some(payload) = error.as_error_resp() {
        // JSON-RPC error `data` is usually a `"0x…"` revert payload (sometimes an
        // object — decode whatever we can parse).
        if let Some(bytes) = payload
            .data
            .as_ref()
            .and_then(|raw| serde_json::from_str::<alloy::primitives::Bytes>(raw.get()).ok())
        {
            return ProviderError::Revert(RevertReason::decode(&bytes));
        }
    }
    ProviderError::Rpc(Box::new(error))
}
