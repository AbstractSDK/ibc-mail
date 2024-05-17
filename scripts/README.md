# IBC Mail Scripts

## Deploying to Rollkit

### Deploy
```
cargo run --bin manual_deploy -- --network-id celeswasm --chain-id celeswasm --address-prefix wasm --grpc-url http://ec2-100-25-222-131.compute-1.amazonaws.com:9290 --gas-denom uwasm
```

### Connect
```
cargo run --bin connect_ibc -- --chain-id constantine-3
```