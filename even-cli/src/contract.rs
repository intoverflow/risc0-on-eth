use alloy_primitives::{FixedBytes, U256};
use alloy_sol_types::{sol, SolInterface};

sol! {
    interface IEvenNumber {
        function set(uint256 x, bytes calldata seal, bytes32 post_state_digest);
    }
}

pub fn set(x: U256, seal: Vec<u8>, post_state_digest: FixedBytes<32>) -> Vec<u8> {
    let calldata = IEvenNumber::IEvenNumberCalls::set(IEvenNumber::setCall {
        x,
        seal,
        post_state_digest,
    });

    calldata.abi_encode()
}
