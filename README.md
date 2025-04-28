# Chainlink Data Streams Solana

This repo contains the on-chain programs + utilities for Chainlink Data Streams on Solana.

## Programs
### Verifier
The on-chain program used to verify data streams DON reports. For more information see the [verifier program documentation](programs/verifier/README.md).


## Developing
This project uses the below tools:
```text
solana-cli 1.18.26
anchor 0.29.0
rustc 1.82.0
```

## Generate Go bindings
Generate the bindings if contract interface is changed

```bash          
go install github.com/gagliardetto/anchor-go@v0.3.1
./client/scripts/anchor-go-gen.sh``
```

