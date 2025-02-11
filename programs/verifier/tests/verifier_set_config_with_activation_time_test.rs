use crate::common::test_setup::{VerifierTestSetup, VerifierTestSetupBuilder};
use solana_program_test::tokio;
use solana_sdk::signature::{Keypair, Signer};
use test_utils::assert::Assert;
use test_utils::report::{generate_report_with_signers, V3Report};
use verifier::common::MAX_NUMBER_OF_ORACLES;
use verifier::errors::ErrorCode;
use verifier::events::ConfigSet;
use verifier::state::VerifierAccount;
use verifier::util::LogParser;
use verifier::state::MAX_NUMBER_OF_DON_CONFIGS;

pub mod common;

#[tokio::test]
async fn set_config_with_activation_time_success_test() {
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

    let mut test_signers: Vec<[u8; 20]> = Vec::new();
    for i in 0..16 {
        let mut signer = [0u8; 20];
        signer[0] = (i + 1) as u8; 
        test_signers.push(signer);
    }

    let f: u8 = 5;
    
    let activation_time: u32 = 1_600_000_000;

    // 6. Call set_config_with_activation_time
    let result = verifier_client
        .set_config_with_activation_time(
            &mut environment_context,
            &user,
            test_signers.clone(),
            f,
            activation_time,
        )
        .await
        .unwrap();

    assert!(
        result.result.is_ok(),
        "set_config_with_activation_time returned an error: {:?}",
        result.result.err()
    );

    // 7. Load the verifier account state and deserialize it
    let verifier_account: VerifierAccount = verifier_client
        .read_verifier_account(&mut environment_context)
        .await
        .unwrap();

    // 8. Verify that the new DonConfig was added correctly
    assert!(
        !verifier_account.don_configs.is_empty(),
        "DonConfigs should not be empty"
    );

    let latest_don_config = verifier_account.don_configs.last().unwrap();

    // Check the fault tolerance parameter
    assert_eq!(
        latest_don_config.f, f,
        "Fault tolerance parameter 'f' does not match"
    );

    // Check the activation time
    assert_eq!(
        latest_don_config.activation_time, activation_time,
        "Activation time does not match"
    );

    // Check that the signers are correctly populated
    for (i, signer) in test_signers.iter().enumerate() {
        assert_eq!(
            latest_don_config.signers[i].key, *signer,
            "Signer at index {} does not match",
            i
        );
    }

    // Check that the remaining signers are set to default ([0u8; 20])
    for signer in latest_don_config.signers.iter().skip(test_signers.len()) {
        assert_eq!(
            signer.key, [0u8; 20],
            "Remaining signers should be default [0u8; 20]"
        );
    }

    assert_eq!(
        latest_don_config.signers.len(), test_signers.len(),
        "Signers should be same size as test signers"
    );

    assert_eq!(
        latest_don_config.signers.capacity() as u8, MAX_NUMBER_OF_ORACLES,
        "Signers should be max size of allowed signers"
    );

    // on success emit event
    // Get the logs from the tx
    let logs: Option<ConfigSet> = LogParser::parse_logs(result.metadata.unwrap().log_messages);
    assert!(logs.is_some(), "Logs should be present");

    let logs = logs.unwrap();

    assert_eq!(
        logs.don_config_id,
        "80ac182049e96c9dbcbd806ba6034cac25ba9a3aa29271e8"
    );
    assert_eq!(logs.signers, test_signers);
    assert_eq!(logs.f, f);
    assert_eq!(
        logs.don_config_index,
        verifier_account.don_configs.len() as u16 - 1
    );
}

#[tokio::test]
async fn set_config_with_activation_time_fails_on_duplicate_don_config_test() {
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

    // 5. Test signers
    let signers: Vec<[u8; 20]> = vec![
        [0x12; 20], // Signer 1
        [0x34; 20], // Signer 2
        [0x56; 20], // Signer 3
        [0x78; 20], // Signer 4
    ];

    // 6. Call set_config_with_activation_time
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

    let verifier_account: VerifierAccount = verifier_client
        .read_verifier_account(&mut environment_context)
        .await
        .unwrap();

    assert!(
        !verifier_account.don_configs.is_empty(),
        "DonConfigs should not be empty"
    );

    // Call set_config_with_activation_time again with the same signers + f. Should fail.
    let result = verifier_client
        .set_config_with_activation_time(
            &mut environment_context,
            &user,
            signers.clone(),
            1,             // f
            1_700_000_000, // notably different activation time
        )
        .await
        .unwrap();

    assert!(
        result.result.is_err(),
        "set_config_with_activation_time should have failed due to duplicate signers"
    );
}

#[tokio::test]
async fn set_config_with_activation_time_fails_on_too_many_signers_test() {
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

    // 5. Test signers -  32 is too many
    let mut signers: Vec<[u8; 20]> = Vec::new();
    for i in 0..32 {
        let signer = [i as u8; 20]; // Create a signer with the value of `i` repeated 20 times
        signers.push(signer);
    }

    // 6. Call set_config_with_activation_time
    let result = verifier_client
        .set_config_with_activation_time(
            &mut environment_context,
            &user,
            signers.clone(),
            2,      // f
            1_600_000_000,
        )
        .await;

    Assert::transaction_error(&result, ErrorCode::ExcessSigners);
}

#[tokio::test]
async fn set_config_with_activation_time_fails_on_too_high_f_test() {
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

    let mut signers: Vec<[u8; 20]> = Vec::new();
    for i in 0..10 {
        let signer = [i as u8; 20];
        signers.push(signer);
    }

    let result = verifier_client
        .set_config_with_activation_time(
            &mut environment_context,
            &user,
            signers.clone(),
            86, // internal calc (86 * 3 = 258) > u8 (256) would overflow the u8
            1_600_000_000,
        )
        .await;

    Assert::transaction_error(&result, ErrorCode::InsufficientSigners);
}

#[tokio::test]
async fn set_config_with_activation_time_fails_on_zero_fault_tolerance_test() {
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

    let mut signers: Vec<[u8; 20]> = Vec::new();
    for i in 0..15 {
        let signer = [i as u8; 20]; // Create a signer with the value of `i` repeated 20 times
        signers.push(signer);
    }

    let result = verifier_client
        .set_config_with_activation_time(
            &mut environment_context,
            &user,
            signers.clone(),
            0,      // f
            1_600_000_000,
        )
        .await;

    Assert::transaction_error(&result, ErrorCode::FaultToleranceMustBePositive);
}

#[tokio::test]
async fn set_config_with_activation_time_fails_on_duplicate_signer_test() {
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

    let signers: Vec<[u8; 20]> = vec![
        [0x12; 20], // Signer 1
        [0x34; 20], // Signer 2
        [0x56; 20], // Signer 3
        [0x56; 20], // Signer 3 -- duplicate
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

    Assert::transaction_error(&result, ErrorCode::NonUniqueSignatures);
}

#[tokio::test]
async fn set_config_with_activation_time_fails_future_activation_time_test() {
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

    let signers: Vec<[u8; 20]> = vec![
        [0x12; 20], // Signer 1
        [0x34; 20], // Signer 2
        [0x56; 20], // Signer 3
        [0x78; 20], // Signer 4
    ];

    let result = verifier_client
        .set_config_with_activation_time(
            &mut environment_context,
            &user,
            signers.clone(),
            1,             // f
            4_000_000_000, // invalid timestamp - future timestamp (year 2096)
        )
        .await;

    Assert::transaction_error(&result, ErrorCode::BadActivationTime);
}

#[tokio::test]
async fn set_config_with_activation_time_fails_on_activation_time_less_than_latest_activation_time_test(
) {
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

    // 5. Test signers
    let signers_1: Vec<[u8; 20]> = vec![
        [0x12; 20], // Signer 1
        [0x34; 20], // Signer 2
        [0x56; 20], // Signer 3
        [0x78; 20], // Signer 4
    ];

    // 6. Call set_config_with_activation_time
    let result = verifier_client
        .set_config_with_activation_time(
            &mut environment_context,
            &user,
            signers_1.clone(),
            1,      // f
            1_600_000_000,
        )
        .await;

    Assert::transaction_ok(&result);

    let verifier_account: VerifierAccount = verifier_client
        .read_verifier_account(&mut environment_context)
        .await
        .unwrap();

    assert!(
        !verifier_account.don_configs.is_empty(),
        "DonConfigs should not be empty"
    );

    let signers_2: Vec<[u8; 20]> = vec![
        [0x22; 20], // Signer 1
        [0x44; 20], // Signer 2
        [0x66; 20], // Signer 3
        [0x88; 20], // Signer 4
    ];

    // Call set_config_with_activation_time again with the same signers + f. Should fail.
    let result = verifier_client
        .set_config_with_activation_time(
            &mut environment_context,
            &user,
            signers_2.clone(),
            1,             // f
            1_500_000_000, // Activation time to less than the previous activation time
        )
        .await;

    Assert::transaction_error(&result, ErrorCode::BadActivationTime);
}

#[tokio::test]
async fn test_set_config_with_activation_time_with_non_owner() {
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
        .set_config_with_activation_time(
            &mut environment_context,
            &non_owner,
            vec![[0; 20]],
            1,
            1_600_000_000
        )
        .await;

    Assert::transaction_error(&result, ErrorCode::Unauthorized);
}


#[tokio::test]
async fn test_setting_duplicate_config_with_back_to_back_signers() {
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

    // Try to set the same config again with a later activation time
    let result = verifier_client
        .set_config_with_activation_time(
            &mut environment_context,
            &user,
            signers_a.clone(),
            f_a,
            1_600_000_001,
        )
        .await;

    // This should fail due to duplicate DON config
    Assert::transaction_error(&result, ErrorCode::DonConfigAlreadyExists);
}


#[tokio::test]
async fn test_set_config_with_signers_containing_zero_address() {
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

    // Generate signers
    let (_, mut signers) = generate_report_with_signers::<V3Report>(16, 6, None, None);
    let f = (signers.len() / 3) as u8;

    // Add zero address
    signers.push([0u8; 20]);

    // Try to set config with zero address
    let result = verifier_client
        .set_config_with_activation_time(
            &mut environment_context,
            &user,
            signers.clone(),
            f,
            1_600_000_000,
        )
        .await;

    Assert::transaction_error(&result, ErrorCode::ZeroAddress);
}


#[tokio::test]
async fn test_setting_duplicate_config_not_back_to_back() {
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

    // Generate different signers for config B
    let (_, signers_b) = generate_report_with_signers::<V3Report>(16, 6, None, None);
    let f_b = (signers_b.len() / 3) as u8;

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

    // Set config B
    let result = verifier_client
        .set_config_with_activation_time(
            &mut environment_context,
            &user,
            signers_b,
            f_b,
            1_600_000_001,
        )
        .await;
    Assert::transaction_ok(&result);

    // Set config A again
    let result = verifier_client
        .set_config_with_activation_time(
            &mut environment_context,
            &user,
            signers_a,
            f_a,
            1_600_000_002,
        )
        .await;
    Assert::transaction_ok(&result);
}


#[tokio::test]
async fn test_set_config_with_f_too_high() {
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

    // Generate signers
    let (_, signers) = generate_report_with_signers::<V3Report>(16, 6, None, None);

    // Try to set config with invalid f
    let invalid_f = 6;
    let result = verifier_client
        .set_config_with_activation_time(
            &mut environment_context,
            &user,
            signers.clone(),
            invalid_f,
            1_600_000_000,
        )
        .await;
    Assert::transaction_error(&result, ErrorCode::InsufficientSigners);

}

#[tokio::test]
async fn test_verify_emits_correct_don_config_id() {
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

    // Create fixed set of 16 signers
    let signers: Vec<[u8; 20]> = (1..17)
        .map(|i| {
            let mut signer = [0u8; 20];
            signer[19] = i as u8;
            signer
        })
        .collect();

    let f = 5; // Set f value
    
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

    // Get the logs from the tx
    let logs: Option<ConfigSet> = LogParser::parse_logs(result.unwrap().metadata.unwrap().log_messages);
    assert!(logs.is_some(), "Logs should be present");

    let logs = logs.unwrap();

    // Check don_config_id matches expected value from Solidity
    assert_eq!(
        logs.don_config_id,
        "56a39dda91c8613fb4720b757cc603299afbcb36340a1cf7" 
    );
}

#[tokio::test]
async fn test_setting_maximum_configs_and_above_max() {
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


    // Set MAX_NUMBER_OF_DON_CONFIGS configs except the last one which will be set later
    for i in 0..(MAX_NUMBER_OF_DON_CONFIGS - 1) as u32 {
        // Generate signers using generate_report_with_signers
        let (_, signers) = generate_report_with_signers::<V3Report>(16, 0, None, None);

        let result = verifier_client
            .set_config_with_activation_time(
                &mut environment_context,
                &user,
                signers.clone(),
                4,      // f
                1_600_000_000 + i, // Increment activation time to make each config unique
            )
            .await;

        Assert::transaction_ok(&result);
    }

    // Generate signers for the final config
    let (_, signers) = generate_report_with_signers::<V3Report>(16, 0, None, None);

    // Set the max config
    let result = verifier_client
        .set_config_with_activation_time(
            &mut environment_context,
            &user,
            signers.clone(),
            4,      // f
            1_600_000_000 + (MAX_NUMBER_OF_DON_CONFIGS as u32), // Final activation time
        )
        .await;

    Assert::transaction_ok(&result);

    // Verify the last config was set correctly
    let verifier_account: VerifierAccount = verifier_client
        .read_verifier_account(&mut environment_context)
        .await
        .unwrap();

    assert_eq!(
        verifier_account.don_configs.len() as usize,
        MAX_NUMBER_OF_DON_CONFIGS,
        "Should have maximum number of configs"
    );

    // Sort the signers to compare 
    let mut sorted_signers = signers.clone();
    sorted_signers.sort();


    // Verify the configs match as a quick sanity check
    let last_config = verifier_account.don_configs.last().unwrap();
    assert_eq!(last_config.f, 4);
    assert_eq!(last_config.activation_time, 1_600_000_000 + (MAX_NUMBER_OF_DON_CONFIGS as u32));
    
    for (i, signer) in sorted_signers.iter().enumerate() {
        assert_eq!(
            last_config.signers[i].key,
            *signer,
            "Signer at index {} does not match",
            i
        );
    }

    // Try to add one more config beyond the max
    let (_, signers) = generate_report_with_signers::<V3Report>(16, 0, None, None);
    
    let result = verifier_client
        .set_config_with_activation_time(
            &mut environment_context,
            &user,
            signers.clone(),
            4,      // f
            1_600_000_000 + (MAX_NUMBER_OF_DON_CONFIGS as u32), 
        )
        .await;
   
    // Should fail with MaxNumberOfConfigsReached error
    Assert::transaction_error(&result, ErrorCode::MaxNumberOfConfigsReached);
}