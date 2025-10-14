// SPDX-License-Identifier: MIT
pragma solidity ^0.8.17;

/// @title ProofRegistry
/// @notice Minimal anchoring contract for PantherSDK proofs.
/// Stores a mapping of proof hashes (bytes32) that were anchored on-chain.
contract ProofRegistry {
    event Anchored(bytes32 indexed proofHash, address indexed sender, uint256 timestamp);

    mapping(bytes32 => bool) public anchored;

    /// @notice Anchor a proof hash. Idempotent: re-anchoring the same hash is allowed.
    function anchor(bytes32 h) external {
        anchored[h] = true;
        emit Anchored(h, msg.sender, block.timestamp);
    }

    /// @notice Check if a proof hash is anchored.
    function isAnchored(bytes32 h) external view returns (bool) {
        return anchored[h];
    }
}

