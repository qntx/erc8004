//! Contract bindings generated via inline Solidity interfaces.
//!
//! These interfaces are sourced from the official ERC-8004 specification
//! (<https://eips.ethereum.org/EIPS/eip-8004>) and the deployed contracts
//! at <https://github.com/erc-8004/erc-8004-contracts>.
//!
//! Using inline Solidity is the alloy-recommended best practice as it
//! preserves full type information (visibility, struct names, etc.) that
//! JSON ABI files omit.

use alloy::sol;

sol! {
    /// ERC-8004 Identity Registry — ERC-721 with `URIStorage` for agent identity.
    ///
    /// Deployed on Ethereum, Base, Polygon, Arbitrum, Celo mainnet at
    /// `0x8004A169FB4a3325136EB29fA0ceB6D2e539a432`.
    #[sol(rpc)]
    contract IdentityRegistry {
        struct MetadataEntry {
            string metadataKey;
            bytes metadataValue;
        }

        event Registered(uint256 indexed agentId, string agentURI, address indexed owner);
        event URIUpdated(uint256 indexed agentId, string newURI, address indexed updatedBy);
        event MetadataSet(uint256 indexed agentId, string indexed indexedMetadataKey, string metadataKey, bytes metadataValue);
        event Transfer(address indexed from, address indexed to, uint256 indexed tokenId);
        event Approval(address indexed owner, address indexed approved, uint256 indexed tokenId);
        event ApprovalForAll(address indexed owner, address indexed operator, bool approved);

        // Registration (three overloads)
        function register() external returns (uint256 agentId);
        function register(string agentURI) external returns (uint256 agentId);
        function register(string agentURI, MetadataEntry[] calldata metadata) external returns (uint256 agentId);

        // URI, wallet & metadata
        function setAgentURI(uint256 agentId, string newURI) external;
        function setAgentWallet(uint256 agentId, address newWallet, uint256 deadline, bytes calldata signature) external;
        function unsetAgentWallet(uint256 agentId) external;
        function setMetadata(uint256 agentId, string metadataKey, bytes metadataValue) external;
        function getMetadata(uint256 agentId, string metadataKey) external view returns (bytes);

        // Queries
        function getAgentWallet(uint256 agentId) external view returns (address);
        function isAuthorizedOrOwner(address spender, uint256 agentId) external view returns (bool);
        function getVersion() external pure returns (string);

        // ERC-721
        function ownerOf(uint256 tokenId) external view returns (address);
        function balanceOf(address owner) external view returns (uint256);
        function tokenURI(uint256 tokenId) external view returns (string);
        function name() external view returns (string);
        function symbol() external view returns (string);
        function approve(address to, uint256 tokenId) external;
        function getApproved(uint256 tokenId) external view returns (address);
        function setApprovalForAll(address operator, bool approved) external;
        function isApprovedForAll(address owner, address operator) external view returns (bool);
        function transferFrom(address from, address to, uint256 tokenId) external;
        function safeTransferFrom(address from, address to, uint256 tokenId) external;
        function safeTransferFrom(address from, address to, uint256 tokenId, bytes data) external;
        function supportsInterface(bytes4 interfaceId) external view returns (bool);

        // EIP-712
        function eip712Domain() external view returns (bytes1 fields, string name, string version, uint256 chainId, address verifyingContract, bytes32 salt, uint256[] extensions);
    }
}

sol! {
    /// ERC-8004 Reputation Registry — feedback and aggregation.
    ///
    /// Deployed on Ethereum, Base, Polygon, Arbitrum, Celo mainnet at
    /// `0x8004BAa17C55a88189AE136b182e5fdA19dE9b63`.
    #[allow(clippy::too_many_arguments, reason = "sol! macro mirrors on-chain ABI")]
    #[sol(rpc)]
    contract ReputationRegistry {
        event NewFeedback(uint256 indexed agentId, address indexed clientAddress, uint64 feedbackIndex, int128 value, uint8 valueDecimals, string indexed indexedTag1, string tag1, string tag2, string endpoint, string feedbackURI, bytes32 feedbackHash);
        event FeedbackRevoked(uint256 indexed agentId, address indexed clientAddress, uint64 indexed feedbackIndex);
        event ResponseAppended(uint256 indexed agentId, address indexed clientAddress, uint64 feedbackIndex, address indexed responder, string responseURI, bytes32 responseHash);

        // Mutations
        function giveFeedback(uint256 agentId, int128 value, uint8 valueDecimals, string tag1, string tag2, string endpoint, string feedbackURI, bytes32 feedbackHash) external;
        function revokeFeedback(uint256 agentId, uint64 feedbackIndex) external;
        function appendResponse(uint256 agentId, address clientAddress, uint64 feedbackIndex, string responseURI, bytes32 responseHash) external;

        // Queries
        function readFeedback(uint256 agentId, address clientAddress, uint64 feedbackIndex) external view returns (int128 value, uint8 valueDecimals, string tag1, string tag2, bool isRevoked);
        function readAllFeedback(uint256 agentId, address[] clientAddresses, string tag1, string tag2, bool includeRevoked) external view returns (address[] clients, uint64[] feedbackIndexes, int128[] values, uint8[] valueDecimals, string[] tag1s, string[] tag2s, bool[] revokedStatuses);
        function getSummary(uint256 agentId, address[] clientAddresses, string tag1, string tag2) external view returns (uint64 count, int128 summaryValue, uint8 summaryValueDecimals);
        function getClients(uint256 agentId) external view returns (address[]);
        function getLastIndex(uint256 agentId, address clientAddress) external view returns (uint64);
        function getResponseCount(uint256 agentId, address clientAddress, uint64 feedbackIndex, address[] responders) external view returns (uint64 count);
        function getIdentityRegistry() external view returns (address);
        function getVersion() external pure returns (string);
    }
}

sol! {
    /// ERC-8004 Validation Registry — validation request/response.
    ///
    /// **Note:** The official deployment address has not yet been listed in the
    /// ERC-8004 contracts README.
    #[sol(rpc)]
    contract ValidationRegistry {
        event ValidationRequest(address indexed validatorAddress, uint256 indexed agentId, string requestURI, bytes32 indexed requestHash);
        event ValidationResponse(address indexed validatorAddress, uint256 indexed agentId, bytes32 indexed requestHash, uint8 response, string responseURI, bytes32 responseHash, string tag);

        // Mutations
        function validationRequest(address validatorAddress, uint256 agentId, string requestURI, bytes32 requestHash) external;
        function validationResponse(bytes32 requestHash, uint8 response, string responseURI, bytes32 responseHash, string tag) external;

        // Queries
        function getValidationStatus(bytes32 requestHash) external view returns (address validatorAddress, uint256 agentId, uint8 response, bytes32 responseHash, string tag, uint256 lastUpdate);
        function getSummary(uint256 agentId, address[] validatorAddresses, string tag) external view returns (uint64 count, uint8 avgResponse);
        function getAgentValidations(uint256 agentId) external view returns (bytes32[]);
        function getValidatorRequests(address validatorAddress) external view returns (bytes32[]);
        function getIdentityRegistry() external view returns (address);
        function getVersion() external pure returns (string);
    }
}
