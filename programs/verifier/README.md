# Verifier Program

Deployed Address: `Gt9S41PtjR58CbG9JhJ3J6vxesqrNAswbWYbLNTMZA3c`

## Integrating with the Verifier Program
There is a lightweight Rust SDK for creating Solana program instructions to verify Chainlink Data Streams reports, supporting both on-chain and off-chain usage.
- Link to the [Rust SDK](crates/chainlink-solana-data-streams/README.md)

If you are using something other than rust you can use the Anchor IDL as a base to interface with the verifier program.
- Link to the [Anchor IDL](target/idl/verifier.json)

The accounts that are required to be passed to the verifier program are:
- The verifier config account (PDA) (The SDK has a utility method to do this for you.)
- The access controller account
- The signer of the transaction
- The report config account (PDA)
    - Derived from the first 32 bytes of the uncompressed report received from the data streams off-chain server.
      The SDK has a utility method to do this for you.

### Integration Examples
- [On-Chain Integration](https://docs.chain.link/data-streams/tutorials/streams-direct/solana-onchain-report-verification)
- [Off-Chain Integration](https://docs.chain.link/data-streams/tutorials/streams-direct/solana-offchain-report-verification)


### Decoding Chainlink Reports
A full rust SDK for data-streams off-chain server is available at https://github.com/smartcontractkit/data-streams-sdk
The SDK contains report schemas, report decoding utilities, and API/Websocket interaction.

## Developing

## Programs
Build the programs by running:

Specific Version to use
```text
solana-cli 1.18.26
anchor 0.29.0
rustc 1.82.0
```

```
anchor build
```

The compiled programs will be in `target/deploy/` folder and the IDL will be in `target/idl/` folder.

Testing the programs

Note: We set a large stack size as tests will use Anchor deserialization of the full account data to validate state.
```bash
RUST_MIN_STACK=16777216 cargo test-sbf
```
