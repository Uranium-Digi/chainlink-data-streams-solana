use crate::common::test_setup::{VerifierTestSetup, VerifierTestSetupBuilder};
use access_controller::AccessController;
use anchor_lang::Discriminator;
use hex::encode as hex_encode;
use hex_literal::hex;
use solana_program::pubkey::Pubkey;
use solana_program_test::tokio;
use solana_sdk::signer::keypair::Keypair;
use solana_sdk::signer::Signer;
use test_utils::report::{generate_report_with_signers, get_signers, DummyReport, V3Report};
use verifier::state::{VerifierAccount, MAX_NUMBER_OF_DON_CONFIGS};
use std::mem::size_of;
use anchor_lang::error::ErrorCode::AccountOwnedByWrongProgram;
use test_utils::assert::Assert;
use verifier::errors::ErrorCode;
use verifier::errors::ErrorCode::{InvalidAccessController, Unauthorized};
use verifier::util::Compressor;

pub mod common;

#[tokio::test]
async fn test_verify_report() {
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

    let test_report_input = hex!("000906f3cbb5a230ad230e8f693aecc4aa5ff7a5c63ecf67ec7201c8a237152c000000000000000000000000000000000000000000000000000000000027018a000000000000000000000000000000000000000000000000000000010000000100000000000000000000000000000000000000000000000000000000000000e000000000000000000000000000000000000000000000000000000000000002200000000000000000000000000000000000000000000000000000000000000280010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001200003ab9412a454b0fb347d0c2c3062186f60640057203d5fb20982d7fb9c927f0000000000000000000000000000000000000000000000000000000067aa71400000000000000000000000000000000000000000000000000000000067aa71400000000000000000000000000000000000000000000000000000221896f269e60000000000000000000000000000000000000000000000000012f260aec4d6000000000000000000000000000000000000000000000000000000000067abc2c00000000000000000000000000000000000000000000000000de08c8d4fa030c80000000000000000000000000000000000000000000000000de065a57df39d200000000000000000000000000000000000000000000000000de0e2fe819758000000000000000000000000000000000000000000000000000000000000000002f173d87393dd8a9dcb97847d1a9a0710e7e4216655477631523a2284e7672790c758cd6413f08d730a3c96b3ede2aca6c9daa554f57b3221754aa579bcea720100000000000000000000000000000000000000000000000000000000000000026284f170f3e580bc532020b1f526b75d7012a632003da5d30f316434689e499509bb8dee47c9c92896e3b2350a74ffcdd564286fc250fe31df3043cfdc2951ef");
    let compressed_report = Compressor::compress(&test_report_input);

    // signers list (as configured when example report was generated)
    let signers: Vec<[u8; 20]> = vec![
        hex!("38C7EA2f6b878509f3e2d0bbE9adF328e1Df2f6C"),
        hex!("a669f0bE9F92e3fe5Eb7b28d1852dFf84C7516Cc"),
        hex!("8735F9dd83c0b03571b39Fe9FfbB05e02bc08c28"),
        hex!("29679cD77AAce065B885b190368f04fDD7E587AD"),
    ];
    let result = verifier_client
        .set_config_with_activation_time(
            &mut environment_context,
            &user,
            signers.clone(),
            1,      // f
            1_600_000_000,
        )
        .await
        .unwrap();
    assert!(
        result.result.is_ok(),
        "set_config_with_activation_time returned an error: {:?}",
        result.result.err()
    );
    // -- End Add Configs

    let result = verifier_client
        .verify(&mut environment_context, &user, compressed_report, None)
        .await;

    Assert::transaction_ok(&result);

    let data = result
        .expect("Transaction result should be present")
        .metadata
        .expect("Metadata should be present")
        .return_data
        .expect("Return data should be present")
        .data;

    let encoded = hex_encode(&data);

    assert_eq!(encoded, "0003ab9412a454b0fb347d0c2c3062186f60640057203d5fb20982d7fb9c927f0000000000000000000000000000000000000000000000000000000067aa71400000000000000000000000000000000000000000000000000000000067aa71400000000000000000000000000000000000000000000000000000221896f269e60000000000000000000000000000000000000000000000000012f260aec4d6000000000000000000000000000000000000000000000000000000000067abc2c00000000000000000000000000000000000000000000000000de08c8d4fa030c80000000000000000000000000000000000000000000000000de065a57df39d200000000000000000000000000000000000000000000000000de0e2fe81975800");
}

#[tokio::test]
async fn test_fail_to_verify_report_if_signer_not_in_config() {
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

    let test_report_input = hex!("000906f3cbb5a230ad230e8f693aecc4aa5ff7a5c63ecf67ec7201c8a237152c000000000000000000000000000000000000000000000000000000000027018a000000000000000000000000000000000000000000000000000000010000000100000000000000000000000000000000000000000000000000000000000000e000000000000000000000000000000000000000000000000000000000000002200000000000000000000000000000000000000000000000000000000000000280010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001200003ab9412a454b0fb347d0c2c3062186f60640057203d5fb20982d7fb9c927f0000000000000000000000000000000000000000000000000000000067aa71400000000000000000000000000000000000000000000000000000000067aa71400000000000000000000000000000000000000000000000000000221896f269e60000000000000000000000000000000000000000000000000012f260aec4d6000000000000000000000000000000000000000000000000000000000067abc2c00000000000000000000000000000000000000000000000000de08c8d4fa030c80000000000000000000000000000000000000000000000000de065a57df39d200000000000000000000000000000000000000000000000000de0e2fe819758000000000000000000000000000000000000000000000000000000000000000002f173d87393dd8a9dcb97847d1a9a0710e7e4216655477631523a2284e7672790c758cd6413f08d730a3c96b3ede2aca6c9daa554f57b3221754aa579bcea720100000000000000000000000000000000000000000000000000000000000000026284f170f3e580bc532020b1f526b75d7012a632003da5d30f316434689e499509bb8dee47c9c92896e3b2350a74ffcdd564286fc250fe31df3043cfdc2951ef");
    let compressed_report = Compressor::compress(&test_report_input);

    // signers list (as configured when example report was generated)
    // but missing signer `29679cD77AAce065B885b190368f04fDD7E587AD`
    let signers: Vec<[u8; 20]> = vec![
        hex!("38C7EA2f6b878509f3e2d0bbE9adF328e1Df2f6C"),
        hex!("a669f0bE9F92e3fe5Eb7b28d1852dFf84C7516Cc"),
        hex!("8735F9dd83c0b03571b39Fe9FfbB05e02bc08c28"),
        hex!("a75F02C31207087dc849007ef9221B11eE6CB559")
    ];

    let result = verifier_client
        .set_config_with_activation_time(
            &mut environment_context,
            &user,
            signers.clone(),
            1,      // f
            1_600_000_000,
        )
        .await;
    Assert::transaction_ok(&result);
    // -- End Add Configs

    let result = verifier_client
        .verify(&mut environment_context, &user, compressed_report, None)
        .await;

    Assert::transaction_error(&result, ErrorCode::BadVerification);
}

#[tokio::test]
async fn test_can_verify_with_non_owner() {
    let non_owner = Keypair::new();

    let VerifierTestSetup {
        mut environment_context,
        verifier_client,
        access_controller_client,
        user,
        ..
    } = VerifierTestSetupBuilder::new()
        .program_name("verifier")
        .program_id(verifier::ID)
        .access_controller(access_controller::ID)
        .add_user(non_owner.pubkey())
        .build()
        .await;

    let result = access_controller_client
        .unwrap()
        .add_access(
            &mut environment_context,
            &user,
            non_owner.pubkey()
            )
            .await;
    Assert::transaction_ok(&result);

    let (report, signers) = generate_report_with_signers::<V3Report>(16, 6, None, None);
    let compressed_report = Compressor::compress(&report);

    // Set up the configuration
    let result = verifier_client
        .set_config_with_activation_time(
            &mut environment_context,
            &user,
            signers.clone(),
            5,      // f
            1_600_000_000,
        )
        .await;
    Assert::transaction_ok(&result);

    // Verify with non-owner
    let result = verifier_client
        .verify(
            &mut environment_context,
            &non_owner,
            compressed_report,
            None,
        )
        .await;

    Assert::transaction_ok(&result);
}


#[tokio::test]
async fn test_can_verify_without_access() {
    let unauthorized_user = Keypair::new();

    let VerifierTestSetup {
        mut environment_context,
        verifier_client,
        user,
        ..
    } = VerifierTestSetupBuilder::new()
        .program_name("verifier")
        .program_id(verifier::ID)
        .access_controller(access_controller::ID)
        .add_user(unauthorized_user.pubkey())
        .build()
        .await;

    let (report, signers) = generate_report_with_signers::<V3Report>(16, 6, None, None);
    let compressed_report = Compressor::compress(&report);

    // Set up the configuration
    let result = verifier_client
        .set_config_with_activation_time(
            &mut environment_context,
            &user,
            signers.clone(),
            5,      // f
            1_600_000_000,
        )
        .await;
    Assert::transaction_ok(&result);

    // Verify with unauthorized_user
    let result = verifier_client
        .verify(
            &mut environment_context,
            &unauthorized_user,
            compressed_report,
            None,
        )
        .await;

    Assert::transaction_error(&result, ErrorCode::Unauthorized);
}

#[tokio::test]
async fn test_verify_fails_with_invalid_access_controller() {
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
        .add_zero_copy_account(access_controller::ID, dummy_access_controller, size_of::<AccessController>(), Some(AccessController::discriminator()))
        .build()
        .await;
    
    // Mock that user will attempt to pass an invalid access controller
    verifier_client.access_controller_data_account_override(Some(dummy_access_controller));
    
    let verifier_account: VerifierAccount = verifier_client
        .read_verifier_account(&mut environment_context)
        .await
        .unwrap();
    
    // Verifier account should have a valid access controller
    assert_ne!(verifier_account.verifier_account_config.access_controller, Pubkey::default());
    assert_ne!(verifier_account.verifier_account_config.access_controller, dummy_access_controller);

    let (report, signers) = generate_report_with_signers::<DummyReport>(16, 6, None, None);
    let compressed_report = Compressor::compress(&report);

    let result = verifier_client
        .verify(&mut environment_context, &user, compressed_report, None)
        .await;
    
    
    Assert::transaction_error(&result, InvalidAccessController);
}

#[tokio::test]
async fn test_incorrect_config_account_fails_verification() {
    let dummy_config_account = Pubkey::new_unique();
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

    let (report, signers) = generate_report_with_signers::<V3Report>(16, 6, None, None);
    let compressed_report = Compressor::compress(&report);

    let result = verifier_client
        .verify(&mut environment_context, &user, compressed_report, Some(dummy_config_account))
        .await;

    Assert::transaction_error(&result, ErrorCode::InvalidConfigAccount);
}


#[tokio::test]
async fn test_fail_to_verify_report_if_not_enough_signers() {
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

    // Generate report with only 1 active signer
    let (report, signers) = generate_report_with_signers::<V3Report>(16, 1, None, None);
    let f = 5; // Requires more than 1 signer
    
    // Set the config first
    let result = verifier_client
        .set_config_with_activation_time(
            &mut environment_context,
            &user,
            signers.clone(),
            f,
            1_600_000_000,
        )
        .await;

    Assert::transaction_ok(&result);

    // Try to verify with insufficient signers
    let compressed_report = Compressor::compress(&report);
    let result = verifier_client
        .verify(&mut environment_context, &user, compressed_report, None)
        .await;

    Assert::transaction_error(&result, ErrorCode::BadVerification);
}

#[tokio::test]
async fn test_fail_to_verify_report_if_no_signers() {
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

    // Generate report with 0 active signers
    let (report, signers) = generate_report_with_signers::<V3Report>(16, 0, None, None);
    let f = 5;
    
    // Set the config first
    let result = verifier_client
        .set_config_with_activation_time(
            &mut environment_context,
            &user,
            signers.clone(),
            f,
            1_600_000_000,
        )
        .await;

    Assert::transaction_ok(&result);

    // Try to verify with no signers
    let compressed_report = Compressor::compress(&report);
    let result = verifier_client
        .verify(&mut environment_context, &user, compressed_report, None)
        .await;

    Assert::transaction_error(&result, ErrorCode::NoSigners);
}

#[tokio::test]
async fn test_fail_to_verify_report_if_dup_signers() {
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

    // Generate signers for the config
    let signers = get_signers(16);
    let signer_pub_keys = signers.iter().map(|s| s.signer_address).collect();
    
    // Add a duplicate signer to generate a malformed report
    let mut signers_with_duplicate = signers[0..6].to_vec();
    signers_with_duplicate.push(signers[0].clone());

    // Generate report with duplicate signers
    let (report, _) = generate_report_with_signers::<V3Report>(16, 7, None, Some(signers_with_duplicate));
    let f = 5;
    
    // Set the config first
    let result = verifier_client
        .set_config_with_activation_time(
            &mut environment_context,
            &user,
            signer_pub_keys,
            f,
            1_600_000_000,
        )
        .await;

    Assert::transaction_ok(&result);

    // Try to verify with duplicate signers
    let compressed_report = Compressor::compress(&report);
    let result = verifier_client
        .verify(&mut environment_context, &user, compressed_report, None)
        .await;

    Assert::transaction_error(&result, ErrorCode::BadVerification);
}

#[tokio::test]
async fn test_verify_with_too_many_signers() {
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

    // Generate report with 8 signers
    let (report, signer_pub_keys) = generate_report_with_signers::<V3Report>(16, 7, None, None);
    let f = 5;
    
    // Set the config
    let result = verifier_client
        .set_config_with_activation_time(
            &mut environment_context,
            &user,
            signer_pub_keys,
            f,
            1_600_000_000,
        )
        .await;

    Assert::transaction_ok(&result);

    // Try to verify with too many signers
    let compressed_report = Compressor::compress(&report);
    let result = verifier_client
        .verify(&mut environment_context, &user, compressed_report, None)
        .await;

    Assert::transaction_ok(&result);
}

#[tokio::test]
async fn test_report_verify_with_latest_config_after_removal() {
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

    // Generate report and signers for config A
    let (report_a, signers_a) = generate_report_with_signers::<V3Report>(16, 6, None, None);
    let f_a = 5;

    // Set config A
    let result = verifier_client
        .set_config_with_activation_time(
            &mut environment_context,
            &user,
            signers_a.clone(),
            f_a,
            1_600_000_000,
        )
        .await;
    Assert::transaction_ok(&result);

    // Generate different signers for config B
    let (_, signers_b) = generate_report_with_signers::<V3Report>(16, 6, None, None);
    let f_b = 5;

    // Set config B with later activation time
    let result = verifier_client
        .set_config_with_activation_time(
            &mut environment_context,
            &user,
            signers_b,
            f_b,
            1_700_000_000,
        )
        .await;
    Assert::transaction_ok(&result);

    // Remove latest config (B)
    let result = verifier_client
        .remove_latest_config(&mut environment_context, &user)
        .await;
    Assert::transaction_ok(&result);

    // Verify report A still works with config A
    let compressed_report = Compressor::compress(&report_a);
    let result = verifier_client
        .verify(&mut environment_context, &user, compressed_report, None)
        .await;

    Assert::transaction_ok(&result);
}

#[tokio::test]
async fn test_can_verify_older_reports_with_older_configs() {
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

    // Generate report and signers for config A with timestamp between config A and B's activation times
    let (report_a, signers_a) = generate_report_with_signers::<V3Report>(16, 6, Some(1_500_000_000), None);
    let f_a = 5;

    // Set config A with activation time 1_400_000_000
    let result = verifier_client
        .set_config_with_activation_time(
            &mut environment_context,
            &user,
            signers_a.clone(),
            f_a,
            1_400_000_000,
        )
        .await;
    Assert::transaction_ok(&result);

    // Generate different signers for config B
    let (_, signers_b) = generate_report_with_signers::<V3Report>(16, 6, None, None);
    let f_b = 5;

    // Set config B with later activation time 1_600_000_000
    let result = verifier_client
        .set_config_with_activation_time(
            &mut environment_context,
            &user,
            signers_b,
            f_b,
            1_600_000_000,
        )
        .await;
    Assert::transaction_ok(&result);

    // Compress the report generated earlier
    let compressed_report = Compressor::compress(&report_a);

    // Verify report - should use config A since report timestamp is before config B's activation
    let result = verifier_client
        .verify(&mut environment_context, &user, compressed_report.clone(), None)
        .await;
    Assert::transaction_ok(&result);

    // Disable config A to ensure it was the config used
    let result = verifier_client
        .set_config_active(&mut environment_context, &user, 0, false)
        .await;
    Assert::transaction_ok(&result);

    // Verify should now fail since config is disabled
    let result = verifier_client
        .verify(&mut environment_context, &user, compressed_report, None)
        .await;
    Assert::transaction_error(&result, ErrorCode::ConfigDeactivated);
    
}

#[tokio::test]
async fn test_can_verify_non_standard_reports() {
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

    // Generate report with signers
    let (report, signers) = generate_report_with_signers::<DummyReport>(16, 6, None, None);
    let compressed_report = Compressor::compress(&report);

    // Set up the configuration
    let result = verifier_client
        .set_config_with_activation_time(
            &mut environment_context,
            &user,
            signers,
            5,      // f
            1_600_000_000,
        )
        .await;
    Assert::transaction_ok(&result);

    // Verify the report
    let result = verifier_client
        .verify(
            &mut environment_context,
            &user,
            compressed_report,
            None,
        )
        .await;

    Assert::transaction_ok(&result);
}
#[tokio::test]
async fn test_rolling_out_configuration() {
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

    // Generate initial signers for config A
    let signers_a = get_signers(16);
    let f = 5;

    // Set config A
    let result = verifier_client
        .set_config_with_activation_time(
            &mut environment_context,
            &user,
            signers_a.iter().map(|s| s.signer_address).collect(),
            f,
            1_600_000_000,
        )
        .await;
    Assert::transaction_ok(&result);

    // Create config B by adding one more signer
    let mut signers_b = signers_a.clone();
    let additional_signer = get_signers(1);
    signers_b.extend(additional_signer);

    // Set config B with later activation time
    let result = verifier_client
        .set_config_with_activation_time(
            &mut environment_context,
            &user,
            signers_b.iter().map(|s| s.signer_address).collect(),
            f,
            1_700_000_000,
        )
        .await;
    Assert::transaction_ok(&result);

    // Generate report using the last 6 signers from config B which includes the new signer
    let report_signers = signers_b[signers_b.len()-6..].to_vec();
    let (report, _) = generate_report_with_signers::<V3Report>(16, 6, None, Some(report_signers));

    // Verify the report
    let compressed_report = Compressor::compress(&report);
    let result = verifier_client
        .verify(&mut environment_context, &user, compressed_report, None)
        .await;

    Assert::transaction_ok(&result);
}

#[tokio::test]
async fn test_report_does_not_verify_when_config_is_disabled() {
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

    // Generate report with signers
    let (report, signers) = generate_report_with_signers::<V3Report>(16, 7, None, None);
    let f = 5;
    
    // Set the config first
    let result = verifier_client
        .set_config_with_activation_time(
            &mut environment_context,
            &user,
            signers.clone(),
            f,
            1_600_000_000,
        )
        .await;

    Assert::transaction_ok(&result);

    // Verify report works initially
    let compressed_report = Compressor::compress(&report);
    let result = verifier_client
        .verify(&mut environment_context, &user, compressed_report.clone(), None)
        .await;

    Assert::transaction_ok(&result);

    // Disable the config by setting it inactive
    let result = verifier_client
        .set_config_active(&mut environment_context, &user, 0, false)
        .await;
    Assert::transaction_ok(&result);

    // Try to verify after disabling
    let result = verifier_client
        .verify(&mut environment_context, &user, compressed_report, None)
        .await;

    Assert::transaction_error(&result, ErrorCode::ConfigDeactivated);
}

#[tokio::test]
async fn test_verify_fails_when_report_is_older_than_config() {
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

    // Generate report with timestamp before config activation time
    let (report, signers) = generate_report_with_signers::<V3Report>(16, 6, Some(1_500_000_000), None);
    let f = 5;

    // Set config with later activation time
    let result = verifier_client
        .set_config_with_activation_time(
            &mut environment_context,
            &user,
            signers,
            f,
            1_600_000_000,
        )
        .await;
    Assert::transaction_ok(&result);

    // Try to verify report from before config was active
    let compressed_report = Compressor::compress(&report);
    let result = verifier_client
        .verify(&mut environment_context, &user, compressed_report, None)
        .await;

    Assert::transaction_error(&result, ErrorCode::BadVerification);
}

#[tokio::test]
async fn test_verify_with_max_configs() {
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

    // Add first config
    let (first_report, first_signers) = generate_report_with_signers::<V3Report>(16, 6, Some(1_600_000_000), None);
    let result = verifier_client
        .set_config_with_activation_time(
            &mut environment_context,
            &user,
            first_signers,
            5,
            1_600_000_000,
        )
        .await;
    Assert::transaction_ok(&result);

    // Add middle configs
    for i in 1..MAX_NUMBER_OF_DON_CONFIGS - 1 {
        let (_, signers) = generate_report_with_signers::<V3Report>(16, 6, None, None);
        let result = verifier_client
            .set_config_with_activation_time(
                &mut environment_context,
                &user,
                signers,
                5,
                1_600_000_000 + i as u32,
            )
            .await;
        Assert::transaction_ok(&result);
    }

    // Add last config
    let (last_report, last_signers) = generate_report_with_signers::<V3Report>(16, 6, Some(1_600_000_000 + MAX_NUMBER_OF_DON_CONFIGS as u32), None);
    let result = verifier_client
        .set_config_with_activation_time(
            &mut environment_context,
            &user,
            last_signers,
            5,
            1_600_000_000 + MAX_NUMBER_OF_DON_CONFIGS as u32,
        )
        .await;
    Assert::transaction_ok(&result);

    // Verify report with first config
    let compressed_first_report = Compressor::compress(&first_report);
    let result = verifier_client
        .verify(&mut environment_context, &user, compressed_first_report, None)
        .await;
    Assert::transaction_ok(&result);

    // Verify report with last config
    let compressed_last_report = Compressor::compress(&last_report);
    let result = verifier_client
        .verify(&mut environment_context, &user, compressed_last_report, None)
        .await;

    Assert::transaction_ok(&result);
}

#[tokio::test]
async fn test_verify_with_max_signers() {
    let VerifierTestSetup {
        mut environment_context,
        user,
        verifier_client,
        ..
    } = VerifierTestSetupBuilder::new()
        .program_name("verifier")
        .program_id(verifier::ID)
        .access_controller(access_controller::ID)
        .with_compute_max_units(2_000_000)
        .build()
        .await;

    // Generate report with 31 signers
    let (report, signers) = generate_report_with_signers::<V3Report>(31, 11, None, None);
    let f = 10; 
    
    // Set the config first
    let result = verifier_client
        .set_config_with_activation_time(
            &mut environment_context,
            &user,
            signers,
            f,
            1_600_000_000,
        )
        .await;

    Assert::transaction_ok(&result);

    // Verify report with max signers
    let compressed_report = Compressor::compress(&report);
    let result = verifier_client
        .verify(&mut environment_context, &user, compressed_report, None)
        .await;

    Assert::transaction_ok(&result);
}

#[tokio::test]
async fn test_verify_with_disabled_access_controller() {
    let unauthorized_user = Keypair::new();
    let authorized_user = Keypair::new();

    let VerifierTestSetup {
        mut environment_context,
        user,
        mut verifier_client,
        access_controller_client,
        access_controller_account_address,
        ..
    } = VerifierTestSetupBuilder::new()
        .program_name("verifier")
        .program_id(verifier::ID)
        .access_controller(access_controller::ID)
        .add_user(unauthorized_user.pubkey())
        .add_user(authorized_user.pubkey())
        .build()
        .await;

    let verifier_account: VerifierAccount = verifier_client
        .read_verifier_account(&mut environment_context)
        .await
        .unwrap();

    // Ensure initial state is access controller enabled with valid accounts
    assert_ne!(access_controller_account_address.unwrap(), Pubkey::default());
    assert_ne!(verifier_account.verifier_account_config.access_controller, Pubkey::default());

    let (report, signers) = generate_report_with_signers::<DummyReport>(16, 6, None, None);
    let compressed_report = Compressor::compress(&report);

    // Set up the configuration
    let result = verifier_client
        .set_config_with_activation_time(
            &mut environment_context,
            &user,
            signers.clone(),
            5,      // f
            1_600_000_000,
        )
        .await;
    Assert::transaction_ok(&result);

    let result = access_controller_client
        .unwrap()
        .add_access(
            &mut environment_context,
            &user,
            authorized_user.pubkey()
        )
        .await;
    Assert::transaction_ok(&result);

    // Authorized user can verify
    let result = verifier_client
        .verify(
            &mut environment_context,
            &authorized_user,
            compressed_report.clone(),
            None,
        )
        .await;
    Assert::transaction_ok(&result);
    
    // Non-access user cannot verify when access controller is enabled
    let result = verifier_client
        .verify(
            &mut environment_context,
            &unauthorized_user,
            compressed_report.clone(),
            None,
        )
        .await;
    Assert::transaction_error(&result, Unauthorized);

    // Disable access controller
    let result = verifier_client
        .set_access_controller(&mut environment_context, &user, None)
        .await;
    Assert::transaction_ok(&result);

    let verifier_account: VerifierAccount = verifier_client
        .read_verifier_account(&mut environment_context)
        .await
        .unwrap();

    // Ensure access controller is disabled in account
    assert_eq!(
        verifier_account.verifier_account_config.access_controller,
        Pubkey::default(),
        "Access controller should be disabled in account (should equal Pubkey::default())"
    );
    
    // When access controller is set we can verify using an existing access controller account even when disabled
    assert_eq!(
        verifier_client.access_controller_data_account.unwrap(),
        access_controller_account_address.unwrap(),
        "Verifier client should use the access controller account address for interaction"
    );

    // Authorized user can verify when access controller is disabled
    let result = verifier_client
        .verify(
            &mut environment_context,
            &authorized_user,
            compressed_report.clone(),
            None,
        )
        .await;
    Assert::transaction_ok(&result);
    
    // Non-access user can verify when access controller is disabled
    let result = verifier_client
        .verify(
            &mut environment_context,
            &unauthorized_user,
            compressed_report.clone(),
            None,
        )
        .await;
    Assert::transaction_ok(&result);
    
    // Transactions with `Pubkey::default()` as AC account will fail during Anchor checks
    verifier_client.access_controller_data_account_override(Some(Pubkey::default()));
    let result = verifier_client
        .verify(
            &mut environment_context,
            &unauthorized_user,
            compressed_report.clone(),
            None,
        )
        .await;
    Assert::transaction_error(&result, AccountOwnedByWrongProgram);

    let result = verifier_client
        .verify(
            &mut environment_context,
            &authorized_user,
            compressed_report.clone(),
            None,
        )
        .await;
    Assert::transaction_error(&result, AccountOwnedByWrongProgram);

    // Set access controller back to valid account
    verifier_client.access_controller_data_account_override(access_controller_account_address);
    assert_eq!(
        verifier_client.access_controller_data_account.unwrap(),
        access_controller_account_address.unwrap(),
        "Verifier client should use the access controller account address for interaction"
    );
    let result = verifier_client
        .set_access_controller(&mut environment_context, &user, access_controller_account_address)
        .await;
    Assert::transaction_ok(&result);

    // Non-access user cannot verify when access controller is re-enabled
    let result = verifier_client
        .verify(
            &mut environment_context,
            &unauthorized_user,
            compressed_report.clone(),
            None,
        )
        .await;
    Assert::transaction_error(&result, Unauthorized);

    // Authorized user can verify when access controller is re-enabled
    let result = verifier_client
        .verify(
            &mut environment_context,
            &authorized_user,
            compressed_report.clone(),
            None,
        )
        .await;
    Assert::transaction_ok(&result);

    // User tries to pass with Any other account will fail

    verifier_client.access_controller_data_account_override(Some(Pubkey::new_unique()));
    let result = verifier_client
        .verify(
            &mut environment_context,
            &unauthorized_user,
            compressed_report.clone(),
            None,
        )
        .await;
    Assert::transaction_error(&result, AccountOwnedByWrongProgram);
    
    verifier_client.access_controller_data_account_override(Some(verifier::ID));
    let result = verifier_client
        .verify(
            &mut environment_context,
            &unauthorized_user,
            compressed_report.clone(),
            None,
        )
        .await;
    Assert::transaction_error(&result, AccountOwnedByWrongProgram);

    verifier_client.access_controller_data_account_override(Some(verifier_client.data_account));
    let result = verifier_client
        .verify(
            &mut environment_context,
            &unauthorized_user,
            compressed_report.clone(),
            None,
        )
        .await;
    Assert::transaction_error(&result, AccountOwnedByWrongProgram);
    
}