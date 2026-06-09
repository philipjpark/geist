// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

/// @title ExecutionGraphRegistry
/// @notice Monad registry for Geist execution-intelligence graph proofs.
contract ExecutionGraphRegistry {
    struct GraphRecord {
        address creator;
        bytes32 signalHash;
        bytes32 graphHash;
        uint256 score;
        string metadataURI;
        uint256 timestamp;
    }

    GraphRecord[] public records;

    event GraphRegistered(
        address indexed creator,
        bytes32 indexed signalHash,
        bytes32 indexed graphHash,
        uint256 score,
        string metadataURI,
        uint256 timestamp
    );

    function registerGraph(
        bytes32 signalHash,
        bytes32 graphHash,
        uint256 score,
        string calldata metadataURI
    ) external returns (uint256 id) {
        require(score <= 100, "score > 100");
        require(signalHash != bytes32(0), "empty signalHash");
        require(graphHash != bytes32(0), "empty graphHash");

        id = records.length;
        records.push(GraphRecord({
            creator: msg.sender,
            signalHash: signalHash,
            graphHash: graphHash,
            score: score,
            metadataURI: metadataURI,
            timestamp: block.timestamp
        }));

        emit GraphRegistered(msg.sender, signalHash, graphHash, score, metadataURI, block.timestamp);
    }

    function recordCount() external view returns (uint256) {
        return records.length;
    }
}
