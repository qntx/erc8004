//! Inline `sol!` interfaces for the three ERC-8004 registries: protocol
//! events, ERC-721 custom errors, and the functions the SDK needs. No
//! `#[sol(rpc)]` — encoding/decoding only.

use alloy::sol;

sol! {
    /// Identity Registry (ERC-721) — events, custom errors, and view/write functions.
    interface IIdentityRegistry {
        #[derive(Debug)]
        event Registered(uint256 indexed agentId, string agentURI, address indexed owner);
        #[derive(Debug)]
        event URIUpdated(uint256 indexed agentId, string newURI, address indexed updatedBy);
        #[derive(Debug)]
        event MetadataSet(uint256 indexed agentId, string indexed indexedMetadataKey, string metadataKey, bytes metadataValue);
        #[derive(Debug)]
        event Transfer(address indexed from, address indexed to, uint256 indexed tokenId);
        #[derive(Debug)]
        event Approval(address indexed owner, address indexed approved, uint256 indexed tokenId);
        #[derive(Debug)]
        event ApprovalForAll(address indexed owner, address indexed operator, bool approved);
        #[derive(Debug)]
        event MetadataUpdate(uint256 _tokenId);
        #[derive(Debug)]
        event BatchMetadataUpdate(uint256 _fromTokenId, uint256 _toTokenId);

        #[derive(Debug)]
        error ERC721NonexistentToken(uint256 tokenId);
        #[derive(Debug)]
        error ERC721IncorrectOwner(address sender, uint256 tokenId, address owner);
        #[derive(Debug)]
        error ERC721InsufficientApproval(address operator, uint256 tokenId);
        #[derive(Debug)]
        error ERC721InvalidReceiver(address receiver);
        #[derive(Debug)]
        error ERC721InvalidSender(address sender);
        #[derive(Debug)]
        error ERC721InvalidOwner(address owner);
        #[derive(Debug)]
        error ERC721InvalidApprover(address approver);
        #[derive(Debug)]
        error ERC721InvalidOperator(address operator);

        function getVersion() view returns (string);
        function register(string agentURI) returns (uint256 agentId);
        function register() returns (uint256 agentId);
        function register(string agentURI, tuple(string,bytes)[] metadata) returns (uint256 agentId);
        function setAgentURI(uint256 agentId, string newURI);
        function getAgentWallet(uint256 agentId) view returns (address);
        function setAgentWallet(uint256 agentId, address newWallet, uint256 deadline, bytes signature);
        function ownerOf(uint256 agentId) view returns (address);
        function tokenURI(uint256 agentId) view returns (string);
        function getApproved(uint256 tokenId) view returns (address);
        function approve(address to, uint256 tokenId);
        function setApprovalForAll(address operator, bool approved);
        function isApprovedForAll(address owner, address operator) view returns (bool);
        function isAuthorizedOrOwner(address spender, uint256 agentId) view returns (bool);
        function getMetadata(uint256 agentId, string metadataKey) view returns (bytes);
        function setMetadata(uint256 agentId, string metadataKey, bytes metadataValue);
        function transferFrom(address from, address to, uint256 tokenId);
        function unsetAgentWallet(uint256 agentId);
    }

    /// Reputation Registry — events and functions.
    interface IReputationRegistry {
        #[derive(Debug)]
        event NewFeedback(uint256 indexed agentId, address indexed clientAddress, uint64 feedbackIndex, int128 value, uint8 valueDecimals, string indexed indexedTag1, string tag1, string tag2, string endpoint, string feedbackURI, bytes32 feedbackHash);
        #[derive(Debug)]
        event FeedbackRevoked(uint256 indexed agentId, address indexed clientAddress, uint64 indexed feedbackIndex);
        #[derive(Debug)]
        event ResponseAppended(uint256 indexed agentId, address indexed clientAddress, uint64 feedbackIndex, address indexed responder, string responseURI, bytes32 responseHash);

        function getVersion() view returns (string);
        function getIdentityRegistry() view returns (address);
        function giveFeedback(uint256 agentId, int128 value, uint8 valueDecimals, string tag1, string tag2, string endpoint, string feedbackURI, bytes32 feedbackHash);
        function revokeFeedback(uint256 agentId, uint64 feedbackIndex);
        function getSummary(uint256 agentId, address[] clientAddresses, string tag1, string tag2) view returns (uint64 count, int128 summaryValue, uint8 summaryValueDecimals);
        function appendResponse(uint256 agentId, address clientAddress, uint64 feedbackIndex, string responseURI, bytes32 responseHash);
        function readFeedback(uint256 agentId, address clientAddress, uint64 feedbackIndex) view returns (int128 value, uint8 valueDecimals, string tag1, string tag2, bool isRevoked);
        function readAllFeedback(uint256 agentId, address[] clientAddresses, string tag1, string tag2, bool includeRevoked) view returns (address[] clients, uint64[] feedbackIndexes, int128[] values, uint8[] valueDecimals, string[] tag1s, string[] tag2s, bool[] revokedStatuses);
        function getClients(uint256 agentId) view returns (address[]);
        function getLastIndex(uint256 agentId, address clientAddress) view returns (uint64);
        function getResponseCount(uint256 agentId, address clientAddress, uint64 feedbackIndex, address[] responders) view returns (uint64 count);
    }

    /// Validation Registry — events and functions.
    interface IValidationRegistry {
        #[derive(Debug)]
        event ValidationRequest(address indexed validatorAddress, uint256 indexed agentId, string requestURI, bytes32 indexed requestHash);
        #[derive(Debug)]
        event ValidationResponse(address indexed validatorAddress, uint256 indexed agentId, bytes32 indexed requestHash, uint8 response, string responseURI, bytes32 responseHash, string tag);

        function getVersion() view returns (string);
        function getIdentityRegistry() view returns (address);
        function validationRequest(address validatorAddress, uint256 agentId, string requestURI, bytes32 requestHash);
        function validationResponse(bytes32 requestHash, uint8 response, string responseURI, bytes32 responseHash, string tag);
        function getValidationStatus(bytes32 requestHash) view returns (address validatorAddress, uint256 agentId, uint8 response, bytes32 responseHash, string tag, uint256 lastUpdate);
        function getSummary(uint256 agentId, address[] validatorAddresses, string tag) view returns (uint64 count, uint8 avgResponse);
        function getAgentValidations(uint256 agentId) view returns (bytes32[]);
        function getValidatorRequests(address validatorAddress) view returns (bytes32[]);
    }
}

#[cfg(test)]
#[allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::tests_outside_test_module,
    reason = "test code panics loudly on failure"
)]
mod tests {
    use alloy::json_abi::JsonAbi;
    use alloy::primitives::keccak256;
    use alloy::sol_types::{Panic, Revert, SolCall, SolError, SolEvent};

    use super::{IIdentityRegistry, IReputationRegistry, IValidationRegistry};

    const IDENTITY: &str = include_str!("../../../data/abis/IdentityRegistry.json");
    const REPUTATION: &str = include_str!("../../../data/abis/ReputationRegistry.json");
    const VALIDATION: &str = include_str!("../../../data/abis/ValidationRegistry.json");

    fn abi(text: &str) -> JsonAbi {
        serde_json::from_str::<JsonAbi>(text).expect("JsonAbi")
    }

    fn check_events(file: &'static str, topics: &[(&str, [u8; 32])]) {
        let abi = abi(file);
        for (name, topic) in topics {
            let events = abi
                .event(name)
                .unwrap_or_else(|| panic!("{file}: event {name} missing from vendored ABI"));
            assert!(
                events
                    .iter()
                    .any(|e| keccak256(e.signature())[..] == topic[..]),
                "{file}: topic mismatch for {name}"
            );
        }
    }

    fn check_functions(file: &'static str, selectors: &[(&str, [u8; 4])]) {
        let abi = abi(file);
        for (name, selector) in selectors {
            let functions = abi
                .function(name)
                .unwrap_or_else(|| panic!("{file}: function {name} missing from vendored ABI"));
            assert!(
                functions
                    .iter()
                    .any(|f| keccak256(f.signature())[..4] == selector[..]),
                "{file}: selector mismatch for {name}"
            );
        }
    }

    fn check_errors(file: &'static str, selectors: &[(&str, [u8; 4])]) {
        let abi = abi(file);
        for (name, selector) in selectors {
            let errors = abi
                .error(name)
                .unwrap_or_else(|| panic!("{file}: error {name} missing from vendored ABI"));
            assert!(
                errors
                    .iter()
                    .any(|e| keccak256(e.signature())[..4] == selector[..]),
                "{file}: selector mismatch for {name}"
            );
        }
    }

    #[test]
    fn identity_registry_matches_vendored_abi() {
        check_events(
            IDENTITY,
            &[
                (
                    "Registered",
                    IIdentityRegistry::Registered::SIGNATURE_HASH.0,
                ),
                (
                    "URIUpdated",
                    IIdentityRegistry::URIUpdated::SIGNATURE_HASH.0,
                ),
                (
                    "MetadataSet",
                    IIdentityRegistry::MetadataSet::SIGNATURE_HASH.0,
                ),
                ("Transfer", IIdentityRegistry::Transfer::SIGNATURE_HASH.0),
                ("Approval", IIdentityRegistry::Approval::SIGNATURE_HASH.0),
                (
                    "ApprovalForAll",
                    IIdentityRegistry::ApprovalForAll::SIGNATURE_HASH.0,
                ),
                (
                    "MetadataUpdate",
                    IIdentityRegistry::MetadataUpdate::SIGNATURE_HASH.0,
                ),
                (
                    "BatchMetadataUpdate",
                    IIdentityRegistry::BatchMetadataUpdate::SIGNATURE_HASH.0,
                ),
            ],
        );
        check_errors(
            IDENTITY,
            &[
                (
                    "ERC721NonexistentToken",
                    IIdentityRegistry::ERC721NonexistentToken::SELECTOR,
                ),
                (
                    "ERC721IncorrectOwner",
                    IIdentityRegistry::ERC721IncorrectOwner::SELECTOR,
                ),
                (
                    "ERC721InsufficientApproval",
                    IIdentityRegistry::ERC721InsufficientApproval::SELECTOR,
                ),
                (
                    "ERC721InvalidReceiver",
                    IIdentityRegistry::ERC721InvalidReceiver::SELECTOR,
                ),
                (
                    "ERC721InvalidSender",
                    IIdentityRegistry::ERC721InvalidSender::SELECTOR,
                ),
                (
                    "ERC721InvalidOwner",
                    IIdentityRegistry::ERC721InvalidOwner::SELECTOR,
                ),
                (
                    "ERC721InvalidApprover",
                    IIdentityRegistry::ERC721InvalidApprover::SELECTOR,
                ),
                (
                    "ERC721InvalidOperator",
                    IIdentityRegistry::ERC721InvalidOperator::SELECTOR,
                ),
            ],
        );
        check_functions(
            IDENTITY,
            &[
                ("getVersion", IIdentityRegistry::getVersionCall::SELECTOR),
                ("register", IIdentityRegistry::register_0Call::SELECTOR),
                ("register", IIdentityRegistry::register_1Call::SELECTOR),
                ("register", IIdentityRegistry::register_2Call::SELECTOR),
                ("setAgentURI", IIdentityRegistry::setAgentURICall::SELECTOR),
                (
                    "getAgentWallet",
                    IIdentityRegistry::getAgentWalletCall::SELECTOR,
                ),
                (
                    "setAgentWallet",
                    IIdentityRegistry::setAgentWalletCall::SELECTOR,
                ),
                ("ownerOf", IIdentityRegistry::ownerOfCall::SELECTOR),
                ("tokenURI", IIdentityRegistry::tokenURICall::SELECTOR),
            ],
        );
    }

    #[test]
    fn reputation_registry_matches_vendored_abi() {
        check_events(
            REPUTATION,
            &[
                (
                    "NewFeedback",
                    IReputationRegistry::NewFeedback::SIGNATURE_HASH.0,
                ),
                (
                    "FeedbackRevoked",
                    IReputationRegistry::FeedbackRevoked::SIGNATURE_HASH.0,
                ),
                (
                    "ResponseAppended",
                    IReputationRegistry::ResponseAppended::SIGNATURE_HASH.0,
                ),
            ],
        );
        check_functions(
            REPUTATION,
            &[
                ("getVersion", IReputationRegistry::getVersionCall::SELECTOR),
                (
                    "getIdentityRegistry",
                    IReputationRegistry::getIdentityRegistryCall::SELECTOR,
                ),
                (
                    "giveFeedback",
                    IReputationRegistry::giveFeedbackCall::SELECTOR,
                ),
                (
                    "revokeFeedback",
                    IReputationRegistry::revokeFeedbackCall::SELECTOR,
                ),
                ("getSummary", IReputationRegistry::getSummaryCall::SELECTOR),
            ],
        );
    }

    #[test]
    fn validation_registry_matches_vendored_abi() {
        check_events(
            VALIDATION,
            &[
                (
                    "ValidationRequest",
                    IValidationRegistry::ValidationRequest::SIGNATURE_HASH.0,
                ),
                (
                    "ValidationResponse",
                    IValidationRegistry::ValidationResponse::SIGNATURE_HASH.0,
                ),
            ],
        );
        check_functions(
            VALIDATION,
            &[
                ("getVersion", IValidationRegistry::getVersionCall::SELECTOR),
                (
                    "getIdentityRegistry",
                    IValidationRegistry::getIdentityRegistryCall::SELECTOR,
                ),
                (
                    "validationRequest",
                    IValidationRegistry::validationRequestCall::SELECTOR,
                ),
                (
                    "validationResponse",
                    IValidationRegistry::validationResponseCall::SELECTOR,
                ),
            ],
        );
    }

    #[test]
    fn standard_error_selectors() {
        assert_eq!(&Revert::SELECTOR, &[0x08, 0xc3, 0x79, 0xa0]);
        assert_eq!(&Panic::SELECTOR, &[0x4e, 0x48, 0x7b, 0x71]);
    }
}
