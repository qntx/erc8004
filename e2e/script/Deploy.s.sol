// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {Script} from "forge-std/Script.sol";
import {console2} from "forge-std/console2.sol";

import {ERC1967Proxy} from "contracts/ERC1967Proxy.sol";
import {HardhatMinimalUUPS} from "contracts/HardhatMinimalUUPS.sol";
import {IdentityRegistryUpgradeable} from "contracts/IdentityRegistryUpgradeable.sol";
import {ReputationRegistryUpgradeable} from "contracts/ReputationRegistryUpgradeable.sol";
import {ValidationRegistryUpgradeable} from "contracts/ValidationRegistryUpgradeable.sol";

/// Replicates the upstream Hardhat test deployment
/// (/tmp/erc8004-contracts/test/core.ts): each registry is a
/// `HardhatMinimalUUPS` behind an `ERC1967Proxy`, upgraded to the real
/// implementation via `upgradeToAndCall` — exercising the production
/// migration path once at deploy time.
contract Deploy is Script {
    function run() external {
        vm.startBroadcast();

        address identity = _deployIdentity();
        address reputation = _deployRegistry(address(new ReputationRegistryUpgradeable()), identity, "reputation");
        address validation = _deployRegistry(address(new ValidationRegistryUpgradeable()), identity, "validation");

        vm.stopBroadcast();

        string memory json = vm.serializeAddress("deployment", "identityRegistry", identity);
        json = vm.serializeAddress("deployment", "reputationRegistry", reputation);
        json = vm.serializeAddress("deployment", "validationRegistry", validation);
        json = vm.serializeUint("deployment", "chainId", block.chainid);
        string memory path = string.concat(vm.projectRoot(), "/../target/e2e/deployment.json");
        vm.writeJson(json, path);
        console2.log("deployment:", path);
    }

    /// HardhatMinimalUUPS → ERC1967Proxy(initialize(address(0))) →
    /// upgradeToAndCall(identityImpl, initialize()).
    function _deployIdentity() internal returns (address) {
        HardhatMinimalUUPS minimal = new HardhatMinimalUUPS();
        ERC1967Proxy proxy = new ERC1967Proxy(
            address(minimal),
            abi.encodeWithSignature("initialize(address)", address(0))
        );
        address impl = address(new IdentityRegistryUpgradeable());
        HardhatMinimalUUPS(address(proxy)).upgradeToAndCall(impl, abi.encodeWithSignature("initialize()"));
        return address(proxy);
    }

    /// HardhatMinimalUUPS → ERC1967Proxy(initialize(identity)) →
    /// upgradeToAndCall(impl, initialize(identity)).
    function _deployRegistry(address impl, address identityRegistry, string memory) internal returns (address) {
        HardhatMinimalUUPS minimal = new HardhatMinimalUUPS();
        bytes memory init = abi.encodeWithSignature("initialize(address)", identityRegistry);
        ERC1967Proxy proxy = new ERC1967Proxy(address(minimal), init);
        HardhatMinimalUUPS(address(proxy)).upgradeToAndCall(impl, init);
        return address(proxy);
    }
}
