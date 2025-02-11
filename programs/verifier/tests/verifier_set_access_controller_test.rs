pub mod common;

use std::mem::size_of;

use crate::common::test_setup::{VerifierTestSetup, VerifierTestSetupBuilder};
use access_controller::AccessController;
use anchor_lang::Discriminator;
use solana_program_test::tokio;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signer};
use test_utils::assert::Assert;
use verifier::errors::ErrorCode;
use verifier::events::AccessControllerSet;
use verifier::state::VerifierAccount;
use verifier::util::LogParser;
use anchor_lang::error::ErrorCode::AccountDiscriminatorMismatch;

#[tokio::test]
async fn set_access_controller() {
    let access_controller_account = Pubkey::new_unique();
    let VerifierTestSetup {
        mut environment_context,
        user,
        verifier_client,
        ..
    } = VerifierTestSetupBuilder::new()
        .program_name("verifier")
        .program_id(verifier::ID)
        .access_controller(access_controller::ID)
        .add_zero_copy_account(access_controller::ID, access_controller_account, size_of::<AccessController>(), Some(AccessController::discriminator()))
        .build()
        .await;

    // Set the access controller
    verifier_client.set_access_controller(&mut environment_context, &user, Some(access_controller_account)).await.unwrap();

    // Load the verifier account state and deserialize it
    let verifier_account_config: VerifierAccount = verifier_client.read_verifier_account(&mut environment_context).await.unwrap();

    // Check the contract account state matches that passed within the instruction
    assert_eq!(access_controller_account, verifier_account_config.verifier_account_config.access_controller);
}

#[tokio::test]
async fn set_access_controller_with_none_disables_access_controller() {
    let access_controller_account = Pubkey::new_unique();
    let VerifierTestSetup {
        mut environment_context,
        user,
        verifier_client,
        ..
    } = VerifierTestSetupBuilder::new()
        .program_name("verifier")
        .program_id(verifier::ID)
        .access_controller(access_controller::ID)
        .add_zero_copy_account(access_controller::ID, access_controller_account, size_of::<AccessController>(), Some(AccessController::discriminator()))
        .build()
        .await;

    // Set the access controller
    verifier_client.set_access_controller(&mut environment_context, &user, None).await.unwrap();

    // Load the verifier account state and deserialize it
    let verifier_account_config: VerifierAccount = verifier_client.read_verifier_account(&mut environment_context).await.unwrap();

    // Default Pubkey is the "disabled" access controller state
    assert_eq!(Pubkey::default(), verifier_account_config.verifier_account_config.access_controller);
}

#[tokio::test]
async fn set_access_controller_with_program_id_disables_access_controller() {
    let access_controller_account = Pubkey::new_unique();
    let VerifierTestSetup {
        mut environment_context,
        user,
        verifier_client,
        ..
    } = VerifierTestSetupBuilder::new()
        .program_name("verifier")
        .program_id(verifier::ID)
        .access_controller(access_controller::ID)
        .add_zero_copy_account(access_controller::ID, access_controller_account, size_of::<AccessController>(), Some(AccessController::discriminator()))
        .build()
        .await;

    // Set the access controller
    verifier_client.set_access_controller(&mut environment_context, &user, Some(verifier::ID)).await.unwrap();

    // Load the verifier account state and deserialize it
    let verifier_account_config: VerifierAccount = verifier_client.read_verifier_account(&mut environment_context).await.unwrap();

    // Default Pubkey is the "disabled" access controller state
    assert_eq!(Pubkey::default(), verifier_account_config.verifier_account_config.access_controller);
}

#[tokio::test]
async fn set_access_controller_emits_event() {
    let access_controller_account = Pubkey::new_unique();
    let VerifierTestSetup {
        mut environment_context,
        user,
        verifier_client,
        ..
    } = VerifierTestSetupBuilder::new()
        .program_name("verifier")
        .program_id(verifier::ID)
        .access_controller(access_controller::ID)
        .add_zero_copy_account(access_controller::ID, access_controller_account, size_of::<AccessController>(), Some(AccessController::discriminator()))
        .build()
        .await;

    // Set the access controller
    let result = verifier_client.set_access_controller(&mut environment_context, &user, Some(access_controller_account)).await.unwrap();

    // Get the logs from the tx
    let logs: Option<AccessControllerSet> = LogParser::parse_logs(result.metadata.unwrap().log_messages);
    assert!(logs.is_some(), "Logs should be present");

    let logs = logs.unwrap();

    assert_eq!(logs.access_controller, access_controller_account, "Access controller should match");
}

#[tokio::test]
async fn test_set_access_controller_with_non_owner() {
    let non_owner = Keypair::new();
    let access_controller_account = Pubkey::new_unique();

    let VerifierTestSetup {
        mut environment_context,
        verifier_client,
        ..
    } = VerifierTestSetupBuilder::new()
        .program_name("verifier")
        .program_id(verifier::ID)
        .access_controller(access_controller::ID)
        .add_zero_copy_account(access_controller::ID, access_controller_account, size_of::<AccessController>(), Some(AccessController::discriminator()))
        .add_user(non_owner.pubkey())
        .build()
        .await;

    let result = verifier_client
        .set_access_controller(&mut environment_context, &non_owner, Some(access_controller_account))
        .await;

     Assert::transaction_error(&result, ErrorCode::Unauthorized);
}

#[tokio::test]
async fn set_invalid_access_controller() {
    let access_controller_dummy_account = Pubkey::new_unique();

    let VerifierTestSetup {
        mut environment_context,
        user,
        verifier_client,
        ..
    } = VerifierTestSetupBuilder::new()
        .program_name("verifier")
        .program_id(verifier::ID)
        .access_controller(access_controller::ID)
        .add_zero_copy_account(access_controller::ID, access_controller_dummy_account, size_of::<AccessController>(), Some(VerifierAccount::discriminator()))
        .build()
        .await;

    // Set the access controller
    let result = verifier_client.set_access_controller(&mut environment_context, &user, Some(access_controller_dummy_account)).await;

    // Transaction should fail as it doesn't have the correct discriminator
    Assert::transaction_error(&result, AccountDiscriminatorMismatch);
}
