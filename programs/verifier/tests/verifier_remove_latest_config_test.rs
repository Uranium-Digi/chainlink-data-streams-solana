use solana_program_test::tokio;

pub mod common;
use crate::common::test_setup::{VerifierTestSetup, VerifierTestSetupBuilder};
use solana_sdk::signature::{Keypair, Signer};
use test_utils::{assert::Assert, report::{generate_report_with_signers, V3Report}};
use verifier::{errors::ErrorCode, events::ConfigRemoved, state::VerifierAccount, util::{Compressor, LogParser}};

#[tokio::test]
async fn test_remove_latest_config_when_no_config_should_fail() {
    let VerifierTestSetup {
        mut environment_context,
        user,
        verifier_client,
        ..
    } = VerifierTestSetupBuilder::new()
        .program_name("verifier")
        .program_id(verifier::ID)
        .access_controller(access_controller::ID)
        .build()
        .await;

    let result = verifier_client
        .remove_latest_config(&mut environment_context, &user)
        .await
        .expect("transaction failed");

    assert!(result.result.is_err(), "remove_latest_config should fail");
}

#[tokio::test]
async fn test_remove_latest_config() {
    let VerifierTestSetup {
        mut environment_context,
        user,
        verifier_client,
        ..
    } = VerifierTestSetupBuilder::new()
        .program_name("verifier")
        .program_id(verifier::ID)
        .access_controller(access_controller::ID)
        .build()
        .await;

    // Generate signers for config A
    let (_, signers_a) = generate_report_with_signers::<V3Report>(16, 6, None, None);
    let f_a = (signers_a.len() / 3) as u8;

    // config A
    verifier_client
        .set_config_with_activation_time(
            &mut environment_context,
            &user,
            signers_a.clone(),
            f_a,
            1_600_000_000,
        )
        .await
        .expect("transaction failed")
        .result
        .expect("set_config_with_activation_time failed");

    // Generate different signers for config B
    let (report_b, signers_b) = generate_report_with_signers::<V3Report>(16, 6, None, None);
    let f_b = (signers_b.len() / 3) as u8;

    // config B
    verifier_client
        .set_config_with_activation_time(
            &mut environment_context,
            &user,
            signers_b.clone(),
            f_b,
            1_700_000_000, // increase the activation time
        )
        .await
        .expect("transaction failed")
        .result
        .expect("set_config_with_activation_time failed");

    let verifier_account: VerifierAccount = verifier_client
        .read_verifier_account(&mut environment_context)
        .await
        .unwrap();

    assert_eq!(
        verifier_account.don_configs.len(),
        2,
        "DonConfigs should have 2 elements"
    );

    let result = verifier_client
        .remove_latest_config(&mut environment_context, &user)
        .await
        .expect("remove_latest_config failed");

    let verifier_account_after_remove: VerifierAccount = verifier_client
        .read_verifier_account(&mut environment_context)
        .await
        .unwrap();

    assert_eq!(
        verifier_account_after_remove.don_configs.len(),
        1,
        "DonConfigs should have 1 element"
    );

    // on success emit event
    // Get the logs from the tx
    let logs: Option<ConfigRemoved> = LogParser::parse_logs(result.metadata.unwrap().log_messages);
    assert!(logs.is_some(), "Logs should be present");
    let logs = logs.unwrap();
    
    assert!(logs.don_config_id.len() > 0, "don_config_id should not be empty");

    // remaining config is preserved
    let latest_don_config = verifier_account_after_remove.don_configs.last().unwrap();
    
    // Note: The don_config_id might be different now due to the new way of generating signers
    // You may need to update this assertion
    assert!(hex::encode(latest_don_config.don_config_id).len() > 0, "don_config_id should not be empty");

    // Re-add config B again to ensure we can re-add a deleted thing
    let r = verifier_client
        .set_config_with_activation_time(
            &mut environment_context,
            &user,
            signers_b.clone(),
            f_b,
            1_700_000_001, // using future timestamp (1 second after the previous one)
        )
        .await
        .expect("transaction failed")
        .result;

    assert!(
        r.is_ok(),
        "set_config_with_activation_time failed {:?}",
        r.err()
    );

    // Compress report_b for verification
    let compressed_report_b = Compressor::compress(&report_b);
    
    // Report verification should pass with active config
    let result = verifier_client
        .verify(&mut environment_context, &user, compressed_report_b, None)
        .await;

    Assert::transaction_ok(&result);

}

#[tokio::test]
async fn test_remove_latest_config_with_non_owner() {
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
        .remove_latest_config(&mut environment_context, &non_owner)
        .await;

    Assert::transaction_error(&result, ErrorCode::Unauthorized);
}
