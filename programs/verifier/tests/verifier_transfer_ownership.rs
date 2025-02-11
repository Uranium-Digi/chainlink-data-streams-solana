use crate::common::test_setup::{VerifierTestSetup, VerifierTestSetupBuilder};
use solana_program_test::tokio;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use test_utils::assert::Assert;
use verifier::errors::ErrorCode;


pub mod common;

#[tokio::test]
async fn test_transfer_ownership_requires_acceptance() {
    let new_owner = Keypair::new();
    let VerifierTestSetup{
        mut environment_context,
        user,
        verifier_client,
        ..
    } = VerifierTestSetupBuilder::new()
        .program_name("verifier")
        .program_id(verifier::ID)
        .access_controller(access_controller::ID)
        .add_user(new_owner.pubkey())
        .build()
        .await;

    // Transfer ownership
    let result = verifier_client
        .transfer_ownership(&mut environment_context, &user, new_owner.pubkey())
        .await;
    Assert::transaction_ok(&result);

    // Verify owner hasn't changed without acceptance
    let verifier_account = verifier_client.read_verifier_account(&mut environment_context).await.unwrap();
    assert_eq!(verifier_account.verifier_account_config.owner, user.pubkey(), "Owner should not change without acceptance");
    assert_eq!(verifier_account.verifier_account_config.proposed_owner, new_owner.pubkey(), "Proposed owner should be set");
}

#[tokio::test]
async fn test_transfer_ownership_and_accept() {
    let new_owner = Keypair::new();
    let VerifierTestSetup{
        mut environment_context,
        user,
        verifier_client,
        ..
    } = VerifierTestSetupBuilder::new()
        .program_name("verifier")
        .program_id(verifier::ID)
        .access_controller(access_controller::ID)
        .add_user(new_owner.pubkey())
        .build()
        .await;

    // Transfer ownership
    let result = verifier_client
        .transfer_ownership(&mut environment_context, &user, new_owner.pubkey())
        .await;
    Assert::transaction_ok(&result);

    // Accept ownership
    let result = verifier_client
        .accept_ownership(&mut environment_context, &new_owner)
        .await;
    Assert::transaction_ok(&result);

    // Verify new owner
    let verifier_account = verifier_client.read_verifier_account(&mut environment_context).await.unwrap();
    assert_eq!(verifier_account.verifier_account_config.owner, new_owner.pubkey(), "Owner should be updated after acceptance");
    assert_eq!(verifier_account.verifier_account_config.proposed_owner, Pubkey::default(), "Proposed owner should be reset after acceptance");

    // Transfer ownership with the old owner should fail
    let result = verifier_client
        .transfer_ownership(&mut environment_context, &user, user.pubkey())
        .await;
    Assert::transaction_error(&result, ErrorCode::Unauthorized);
}

#[tokio::test]
async fn test_transfer_ownership_can_be_called_multiple_times() {
    let first_proposed_owner = Keypair::new();
    let second_proposed_owner = Keypair::new();
    let VerifierTestSetup{
        mut environment_context,
        user,
        verifier_client,
        ..
    } = VerifierTestSetupBuilder::new()
        .program_name("verifier")
        .program_id(verifier::ID)
        .access_controller(access_controller::ID)
        .add_user(first_proposed_owner.pubkey())
        .add_user(second_proposed_owner.pubkey())
        .build()
        .await;

    // First transfer
    let result = verifier_client
        .transfer_ownership(&mut environment_context, &user, first_proposed_owner.pubkey())
        .await;
    Assert::transaction_ok(&result);

    // Second transfer before acceptance
    let result = verifier_client
        .transfer_ownership(&mut environment_context, &user, second_proposed_owner.pubkey())
        .await;
    Assert::transaction_ok(&result);

    // First proposed owner tries to accept (should fail)
    let result = verifier_client
        .accept_ownership(&mut environment_context, &first_proposed_owner)
        .await;
    Assert::transaction_error(&result, ErrorCode::Unauthorized);

    // Second proposed owner accepts
    let result = verifier_client
        .accept_ownership(&mut environment_context, &second_proposed_owner)
        .await;
    Assert::transaction_ok(&result);

    // Verify final owner
    let verifier_account = verifier_client.read_verifier_account(&mut environment_context).await.unwrap();
    assert_eq!(verifier_account.verifier_account_config.owner, second_proposed_owner.pubkey(), "Owner should be the last proposed owner");
    assert_eq!(verifier_account.verifier_account_config.proposed_owner, Pubkey::default(), "Proposed owner should be reset after acceptance");
}

#[tokio::test]
async fn test_only_owner_can_initiate_transfer() {
    let non_owner = Keypair::new();
    let new_owner = Keypair::new();
    let VerifierTestSetup{
        mut environment_context,
        verifier_client,
        ..
    } = VerifierTestSetupBuilder::new()
        .program_name("verifier")
        .program_id(verifier::ID)
        .access_controller(access_controller::ID)
        .add_user(non_owner.pubkey())
        .add_user(new_owner.pubkey())
        .build()
        .await;

    // Non-owner tries to transfer ownership
    let result = verifier_client
        .transfer_ownership(&mut environment_context, &non_owner, new_owner.pubkey())
        .await;
    Assert::transaction_error(&result, ErrorCode::Unauthorized);
}

#[tokio::test]
async fn test_only_proposed_owner_can_accept() {
    let proposed_owner = Keypair::new();
    let non_proposed_owner = Keypair::new();
    let VerifierTestSetup {
        mut environment_context,
        user,
        verifier_client,
        ..
    } = VerifierTestSetupBuilder::new()
        .program_name("verifier")
        .program_id(verifier::ID)
        .access_controller(access_controller::ID)
        .add_user(proposed_owner.pubkey())
        .add_user(non_proposed_owner.pubkey())
        .build()
        .await;

    // Transfer ownership
    let result = verifier_client
        .transfer_ownership(&mut environment_context, &user, proposed_owner.pubkey())
        .await;
    Assert::transaction_ok(&result);

    // Non-proposed owner tries to accept
    let result = verifier_client
        .accept_ownership(&mut environment_context, &non_proposed_owner)
        .await;
    Assert::transaction_error(&result, ErrorCode::Unauthorized);
}
