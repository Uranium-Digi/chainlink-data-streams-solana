use crate::common::test_setup::{VerifierTestSetup, VerifierTestSetupBuilder};
use hex_literal::hex;
use solana_program_test::tokio;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;

use test_utils::assert::Assert;
use verifier::errors::ErrorCode;
use verifier::util::Compressor;

pub mod common;

#[tokio::test]
async fn test_set_config_active_unknown_don_config_id() {
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
        .set_config_active(&mut environment_context, &user, 1, true)
        .await;

    Assert::transaction_error(&result, ErrorCode::DonConfigDoesNotExist);
    
}

#[tokio::test]
async fn test_set_config_active() {
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
    
    // Add configs
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
            1 // timestamp prior to report timestamp
        )
        .await;

    Assert::transaction_ok(&result);
    
    let result = verifier_client
        .set_config_active(&mut environment_context, &user, 0, false)
        .await;
    
    Assert::transaction_ok(&result);
    
    let test_report_input = hex!("000906f3cbb5a230ad230e8f693aecc4aa5ff7a5c63ecf67ec7201c8a237152c000000000000000000000000000000000000000000000000000000000027018a000000000000000000000000000000000000000000000000000000010000000100000000000000000000000000000000000000000000000000000000000000e000000000000000000000000000000000000000000000000000000000000002200000000000000000000000000000000000000000000000000000000000000280010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001200003ab9412a454b0fb347d0c2c3062186f60640057203d5fb20982d7fb9c927f0000000000000000000000000000000000000000000000000000000067aa71400000000000000000000000000000000000000000000000000000000067aa71400000000000000000000000000000000000000000000000000000221896f269e60000000000000000000000000000000000000000000000000012f260aec4d6000000000000000000000000000000000000000000000000000000000067abc2c00000000000000000000000000000000000000000000000000de08c8d4fa030c80000000000000000000000000000000000000000000000000de065a57df39d200000000000000000000000000000000000000000000000000de0e2fe819758000000000000000000000000000000000000000000000000000000000000000002f173d87393dd8a9dcb97847d1a9a0710e7e4216655477631523a2284e7672790c758cd6413f08d730a3c96b3ede2aca6c9daa554f57b3221754aa579bcea720100000000000000000000000000000000000000000000000000000000000000026284f170f3e580bc532020b1f526b75d7012a632003da5d30f316434689e499509bb8dee47c9c92896e3b2350a74ffcdd564286fc250fe31df3043cfdc2951ef");
    let compressed_report = Compressor::compress(&test_report_input);

    // Report verification should fail with inactive config
    let result = verifier_client
        .verify(&mut environment_context, &user, compressed_report.clone(), None)
        .await;
    Assert::transaction_error(&result, ErrorCode::ConfigDeactivated);

    // Set Config Active
    let result = verifier_client
        .set_config_active(&mut environment_context, &user, 0, true)
        .await;
    Assert::transaction_ok(&result);
    
    // Report verification should pass with active config
    let result = verifier_client
        .verify(&mut environment_context, &user, compressed_report, None)
        .await;
    Assert::transaction_ok(&result);
}

#[tokio::test]
async fn test_set_config_active_with_non_owner() {
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
        .set_config_active(&mut environment_context, &non_owner, 0, true)
        .await;

    Assert::transaction_error(&result, ErrorCode::Unauthorized);
}