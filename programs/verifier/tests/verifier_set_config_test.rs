use crate::common::test_setup::{VerifierTestSetup, VerifierTestSetupBuilder};
use solana_program_test::tokio;
use solana_sdk::signature::{Keypair, Signer};
use test_utils::assert::Assert;
use verifier::errors::ErrorCode;

pub mod common;

#[tokio::test]
async fn test_set_config_with_non_owner() {
    let non_owner = Keypair::new();
    let VerifierTestSetup {
        mut environment_context,
        verifier_client,
        ..
    } = VerifierTestSetupBuilder::new()
        .program_name("verifier")
        .program_id(verifier::ID)
        .access_controller(access_controller::ID)
        .add_user(non_owner.pubkey())
        .build()
        .await;

    let result = verifier_client
        .set_config(
            &mut environment_context,
            &non_owner,
            vec![[0; 20]],
            1,
        )
        .await;

    Assert::transaction_error(&result, ErrorCode::Unauthorized);
}
