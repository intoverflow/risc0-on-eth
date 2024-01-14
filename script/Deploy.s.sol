// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import {Script} from "forge-std/Script.sol";
import {console2} from "forge-std/console2.sol";
import {IRiscZeroVerifier} from "bonsai/IRiscZeroVerifier.sol";
import {ControlID, RiscZeroGroth16Verifier} from "bonsai/groth16/RiscZeroGroth16Verifier.sol";

import "../contracts/EvenNumber.sol";

contract EvenNumberDeploy is Script {
    function run() external {
        uint256 deployerKey = vm.envUint("ETH_WALLET_PRIVATE_KEY");
        bytes32 imageId = vm.envBytes32("GUEST_IMAGE_ID");

        console2.log("Guest Image ID is");
        console2.logBytes32(imageId);

        vm.startBroadcast(deployerKey);

        IRiscZeroVerifier verifier = new RiscZeroGroth16Verifier(ControlID.CONTROL_ID_0, ControlID.CONTROL_ID_1);
        console2.log("Deployed RiscZeroGroth16Verifier to", address(verifier));

        EvenNumber evenNumber = new EvenNumber(verifier, imageId);
        console2.log("Deployed EvenNumber to", address(evenNumber));

        vm.stopBroadcast();
    }
}