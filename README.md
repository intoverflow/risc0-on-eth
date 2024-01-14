# risc0-on-eth
An example of how to use RISC Zero's zkVM to protect an Ethereum smart contract

The [`EvenNumber`](https://github.com/intoverflow/risc0-on-eth/blob/main/contracts/EvenNumber.sol) contract wraps a `uint256` and provides `set()` and `get()` functions. The contract guarantees that the wrapped number is always even.

To enforce this guarantee, the contract's `set()` function requires a ZK proof that the new value is even. The proof is generated by running [`even-guests/is-even`](https://github.com/intoverflow/risc0-on-eth/blob/main/even-guests/is-even/src/main.rs) from within the RISC Zero zkVM.

An example of how to tie everything together is given by the [`even-cli`](https://github.com/intoverflow/risc0-on-eth/blob/main/even-cli/src/main.rs) tool, which gives us a way to invoke the contract's `set()` function from the command line. This tool:

* Uses Bonsai to generate the ZK proof.
* Sends a transaction (with the appropriate calldata) to the contract via an RPC provider.

Example usage is given below.

## Get the tools

You'll need [foundry](https://github.com/foundry-rs/foundry) and [risc0](https://github.com/risc0/risc0/).

## Clone the repo and fetch the dependencies

```console
$ git clone https://github.com/intoverflow/risc0-on-eth.git
$ cd risc0-on-eth
$ git submodule update --init --recursive
```

## Test with forge

```console
$ forge test -vvvvv
```

## Test with anvil

### Start the `anvil` service

```console
$ anvil -a 1

...

Available Accounts
==================

(0) "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266" (10000.000000000000000000 ETH)

Private Keys
==================

(0) 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80

...

Listening on 127.0.0.1:8545
```

We now save our private key into an environment variable. This will allow us to deploy our contracts and submit transactions.

```console
$ export ETH_WALLET_PRIVATE_KEY=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
```

### Deploy the guest to Bonsai

This requires that `BONSAI_API_URL` and `BONSAI_API_KEY` are set:

```console
$ cargo run --release -- deploy
a233b08506289266e2209d24fee095c44564e97eb303547c25220a7a0cd96757
```

On success, the tool outputs the guest's Image ID. We save this value to an environment variable. This will allow us to deploy our contracts.

```console
$ export GUEST_IMAGE_ID=a233b08506289266e2209d24fee095c44564e97eb303547c25220a7a0cd96757
```

We can optionally test the guest deployment (and our environment variables) at this time:

```console
$ cargo run --release -- \
    test-vector \
    --image-id=${GUEST_IMAGE_ID} \
    --number=12345678
```

This command given above fetches a Snark receipt from Bonsai and print its contents. The output can be used to create [unit tests](https://github.com/intoverflow/risc0-on-eth/blob/main/tests/EvenNumber.sol) for the `EvenNumber` contract.

### Deploy the contracts

This requires that `GUEST_IMAGE_ID` and `ETH_WALLET_PRIVATE_KEY` are set:

```console
$ forge script --rpc-url http://localhost:8545 --broadcast script/Deploy.s.sol

...

== Logs ==
  Guest Image ID is
  0xa233b08506289266e2209d24fee095c44564e97eb303547c25220a7a0cd96757
  Deployed RiscZeroGroth16Verifier to 0x5FbDB2315678afecb367f032d93F642f64180aa3
  Deployed EvenNumber to 0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512

...
```

### Interact with the contract

First, let's query the contract's state:

```console
$ cast call --rpc-url http://localhost:8545 \
    0xe7f1725e7734ce288f8367e1bb143e90bb3f0512 \
    'get()(uint256)'
0
```

Now let's update its state. We can do this by submitting a transaction. This requires that `BONSAI_API_URL`, `BONSAI_API_KEY`, and `ETH_WALLET_PRIVATE_KEY` are set:

```console
$ cargo run --release -- \
    send-tx \
    --image-id=a233b08506289266e2209d24fee095c44564e97eb303547c25220a7a0cd96757 \
    --chain-id=31337 \
    --rpc-url=http://localhost:8545 \
    --contract=e7f1725E7734CE288F8367e1Bb143E90bb3F0512 \
    --number=12345678
```

Lastly, let's query the state one more time to see that the value has been updated:

```console
$ cast call --rpc-url http://localhost:8545 \
    0xe7f1725e7734ce288f8367e1bb143e90bb3f0512 \
    'get()(uint256)'
12345678
```
