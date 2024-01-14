# risc0-on-eth
An example of how to use RISC Zero's zkVM to protect an Ethereum smart contract

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

### Set your private key

```console
$ export ETH_WALLET_PRIVATE_KEY=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
```

### Deploy the contracts

This requires that `ETH_WALLET_PRIVATE_KEY` be set.

```console
$ forge script --rpc-url http://localhost:8545 --broadcast script/Deploy.s.sol

...

== Logs ==
  Deployed RiscZeroGroth16Verifier to 0x5FbDB2315678afecb367f032d93F642f64180aa3
  Deployed EvenNumber to 0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512

...
```

### Query the state

```console
$ cast call --rpc-url http://localhost:8545 \
    0xe7f1725e7734ce288f8367e1bb143e90bb3f0512 \
    'get()(uint256)'
0
```

### Update the number

This requires that `BONSAI_API_URL`, `BONSAI_API_KEY`, and `ETH_WALLET_PRIVATE_KEY` be set.

```console
$ cargo run --release -- \
    --chain-id=31337 \
    --rpc-url=http://localhost:8545 \
    --contract=e7f1725E7734CE288F8367e1Bb143E90bb3F0512 \
    --number=12345678
```

Query the state to see that the value has been updated:

```console
$ cast call --rpc-url http://localhost:8545 \
    0xe7f1725e7734ce288f8367e1bb143e90bb3f0512 \
    'get()(uint256)'
12345678
```
