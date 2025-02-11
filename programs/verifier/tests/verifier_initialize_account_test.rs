pub mod common;

use crate::common::test_setup::{VerifierTestSetup, VerifierTestSetupBuilder};
use access_controller::AccessController;
use anchor_lang::error::ErrorCode::AccountDiscriminatorMismatch;
use anchor_lang::Discriminator;
use solana_program::pubkey::Pubkey;
use solana_program::system_instruction::SystemError;
use solana_program_test::tokio;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use test_utils::assert::Assert;
use verifier::errors::ErrorCode::InvalidInputs;
use verifier::errors::ErrorCode::Unauthorized;
use verifier::state::VerifierAccount;

#[tokio::test]
async fn initialize_test() {
    let VerifierTestSetup {
        mut environment_context,
        verifier_client,
        access_controller_account_address,
        ..
    } = VerifierTestSetupBuilder::new()
        .program_name("verifier")
        .program_id(verifier::ID)
        .access_controller(access_controller::ID)
        .build()
        .await;

    // Load the verifier account state and deserialize it
    let verifier_account: VerifierAccount = verifier_client
        .read_verifier_account(&mut environment_context)
        .await
        .unwrap();

    // Check the contract account state matches that passed within the instruction
    assert_eq!(
        access_controller_account_address.unwrap(),
        verifier_account.verifier_account_config.access_controller
    );
}

#[tokio::test]
async fn initialize_twice_fails() {
    let non_owner = Keypair::new();
    let VerifierTestSetup {
        mut environment_context,
        verifier_client,
        user,
        ..
    } = VerifierTestSetupBuilder::new()
        .program_name("verifier")
        .program_id(verifier::ID)
        .access_controller(access_controller::ID)
        .add_user(non_owner.pubkey())
        .init(false)
        .build()
        .await;

    let result = verifier_client
        .initialize(&mut environment_context, &user)
        .await;

    Assert::transaction_ok(&result);

    let result = verifier_client
        .initialize(&mut environment_context, &user)
        .await;

    Assert::system_error(&result, SystemError::AccountAlreadyInUse);
}

#[tokio::test]
async fn initialize_data_fails_when_instructed_twice() {
    let VerifierTestSetup {
        mut environment_context,
        verifier_client,
        user,
        access_controller_client,
        ..
    } = VerifierTestSetupBuilder::new()
        .program_name("verifier")
        .program_id(verifier::ID)
        .access_controller(access_controller::ID)
        .init(false)
        .build()
        .await;

    let result = access_controller_client
        .unwrap()
        .initialize(&mut environment_context, &user)
        .await;

    Assert::transaction_ok(&result);

    let result = verifier_client
        .initialize(&mut environment_context, &user)
        .await;

    Assert::transaction_ok(&result);

    let result = verifier_client
        .realloc_full_size(&mut environment_context, &user)
        .await;

    Assert::transaction_ok(&result);

    let result = verifier_client
        .init_data(&mut environment_context, &user)
        .await;

    Assert::transaction_ok(&result);

    let result = verifier_client
        .init_data(&mut environment_context, &user)
        .await;

    Assert::transaction_error(&result, InvalidInputs);
}

#[tokio::test]
async fn initialize_fails_with_non_upgrade_authority_test() {
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
        .init(false)
        .build()
        .await;

    let result = verifier_client
        .initialize(&mut environment_context, &non_owner)
        .await;

    Assert::transaction_error(&result, Unauthorized);
}

#[tokio::test]
async fn realloc_fails_with_non_upgrade_authority_test() {
    let non_owner = Keypair::new();
    let VerifierTestSetup {
        mut environment_context,
        verifier_client,
        user,
        ..
    } = VerifierTestSetupBuilder::new()
        .program_name("verifier")
        .program_id(verifier::ID)
        .access_controller(access_controller::ID)
        .add_user(non_owner.pubkey())
        .init(false)
        .build()
        .await;

    let result = verifier_client
        .initialize(&mut environment_context, &user)
        .await;

    Assert::transaction_ok(&result);

    let result = verifier_client
        .realloc(&mut environment_context, &non_owner, 10 * 1024)
        .await;

    Assert::transaction_error(&result, Unauthorized);
}

#[tokio::test]
async fn initialize_account_data_fails_with_non_upgrade_authority_test() {
    let non_owner = Keypair::new();
    let VerifierTestSetup {
        mut environment_context,
        verifier_client,
        user,
        access_controller_client,
        ..
    } = VerifierTestSetupBuilder::new()
        .program_name("verifier")
        .program_id(verifier::ID)
        .access_controller(access_controller::ID)
        .add_user(non_owner.pubkey())
        .init(false)
        .build()
        .await;

    let result = access_controller_client
        .unwrap()
        .initialize(&mut environment_context, &user)
        .await;

    Assert::transaction_ok(&result);

    let result = verifier_client
        .initialize(&mut environment_context, &user)
        .await;

    Assert::transaction_ok(&result);

    let result = verifier_client
        .realloc_full_size(&mut environment_context, &user)
        .await;

    Assert::transaction_ok(&result);

    let result = verifier_client
        .init_data(&mut environment_context, &non_owner)
        .await;

    Assert::transaction_error(&result, Unauthorized);
}

#[tokio::test]
async fn initialize_fails_with_malformed_access_controller_discriminator_test() {
    let dummy_access_controller = Pubkey::new_unique();
    let VerifierTestSetup {
        mut environment_context,
        user,
        mut verifier_client,
        ..
    } = VerifierTestSetupBuilder::new()
        .program_name("verifier")
        .program_id(verifier::ID)
        .access_controller(access_controller::ID)
        .add_zero_copy_account(
            access_controller::ID,
            dummy_access_controller,
            size_of::<AccessController>(),
            Some(VerifierAccount::discriminator()),
        )
        .init(false)
        .build()
        .await;

    verifier_client.access_controller_data_account_override(Some(dummy_access_controller));

    let result = verifier_client
        .initialize_realloc_init_data(&mut environment_context, &user)
        .await;

    Assert::transaction_error(&result, AccountDiscriminatorMismatch);
}
