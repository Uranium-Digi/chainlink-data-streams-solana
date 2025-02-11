use solana_program::instruction::InstructionError;
use solana_program_test::{BanksClientError, BanksTransactionResultWithMetadata};
use solana_sdk::transaction::TransactionError;
use std::any::type_name_of_val;
use solana_program::system_instruction::SystemError;

pub struct Assert {}


impl Assert {
    #[track_caller]
    pub fn transaction_ok(
        result: &Result<BanksTransactionResultWithMetadata, BanksClientError>
    ) {
        // Unwrap the BanksTransactionResultWithMetadata from the result
        let tx_result = match result {
            Ok(tx_result) => tx_result,
            Err(e) => panic!("Transaction processing failed: {:?}", e),
        };

        // Extract the transaction result, which may contain a TransactionError
        let inner_result = &tx_result.result;
        assert!(
            inner_result.is_ok(),
            "Failed unexpectedly: {:?} | error type {}",
            inner_result.clone().err(),
            type_name_of_val(&inner_result.clone().err())
        );
    }


    #[track_caller]
    pub fn transaction_error<E>(
        result: &Result<BanksTransactionResultWithMetadata, BanksClientError>,
        expected_error: E,
    ) where E: Into<u32> {
        // Unwrap the BanksTransactionResultWithMetadata from the result
        let tx_result = match result {
            Ok(tx_result) => tx_result,
            Err(e) => panic!("Transaction processing failed: {:?}", e),
        };

        // Extract the transaction result, which may contain a TransactionError
        let inner_result = &tx_result.result;

        match inner_result {
            Err(transaction_error) => {
                // Match on the TransactionError to extract the InstructionError
                match transaction_error {
                    TransactionError::InstructionError(_, instruction_error) => {
                        match instruction_error {
                            InstructionError::Custom(received_error_code) => {
                                // We use Anchor `error_code` which uses base error code of 6000
                                let expected_error_code: u32 = expected_error.into();
                                assert_eq!(
                                    *received_error_code, expected_error_code,
                                    "Expected error code {} but received {}.",
                                    expected_error_code, received_error_code
                                );
                            }
                            _ => panic!("Unexpected InstructionError: {:?}", instruction_error),
                        }
                    }
                    _ => panic!("Unexpected TransactionError: {:?}", transaction_error),
                }
            }
            Ok(()) => panic!("Expected transaction to fail with an error, but it succeeded."),
        }
    }

    #[track_caller]
    pub fn system_error(
        result: &Result<BanksTransactionResultWithMetadata, BanksClientError>,
        expected_error: SystemError,
    ) {
        // Map SystemError to its corresponding error code
        let expected_code = match expected_error {
            SystemError::AccountAlreadyInUse => 0,
            _ => {
                panic!("Unsupported system error: {:?}. Add the error as needed.", expected_error);
            }
        };

        // Unwrap the BanksTransactionResultWithMetadata from the result
        let tx_result = match result {
            Ok(tx_result) => tx_result,
            Err(e) => panic!("Transaction processing failed: {:?}", e),
        };

        // Extract the transaction result, which may contain a TransactionError
        let inner_result = &tx_result.result;

        match inner_result {
            Err(transaction_error) => {
                // Match on the TransactionError to extract the InstructionError
                match transaction_error {
                    TransactionError::InstructionError(_, instruction_error) => {
                        match instruction_error {
                            InstructionError::Custom(received_code) => {
                                assert_eq!(
                                    *received_code,
                                    expected_code,
                                    "Expected system error {:?} (code: {}) but received code {}",
                                    expected_error,
                                    expected_code,
                                    received_code
                                );
                            }
                            _ => panic!(
                                "Expected system error {:?} (code: {}) but received {:?}",
                                expected_error, expected_code, instruction_error
                            ),
                        }
                    }
                    _ => panic!("Unexpected TransactionError: {:?}", transaction_error),
                }
            }
            Ok(()) => panic!(
                "Expected transaction to fail with system error {:?} (code: {}), but it succeeded.",
                expected_error, expected_code
            ),
        }
    }


}
