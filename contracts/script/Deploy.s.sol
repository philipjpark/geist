// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "forge-std/Script.sol";
import "../src/ExecutionGraphRegistry.sol";

contract Deploy is Script {
    function run() external returns (ExecutionGraphRegistry registry) {
        vm.startBroadcast();
        registry = new ExecutionGraphRegistry();
        vm.stopBroadcast();
    }
}
