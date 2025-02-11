use anchor_lang::prelude::Pubkey;
use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::AccountDeserialize;
use solana_program_test::{BanksClientError, BanksTransactionResultWithMetadata, ProgramTestContext};
use solana_sdk::account::Account;
use solana_sdk::signature::Keypair;
use solana_sdk::transaction::Transaction;
use anchor_lang::solana_program::program_error::ProgramError;

pub struct EnvironmentContextOperations {}

impl EnvironmentContextOperations {
    pub async fn send_transaction(environment_context: &mut ProgramTestContext,
                                  instructions: &[Instruction],
                                  payer: Option<&Pubkey>,
                                  signing_keypairs: &[&Keypair]) -> Result<BanksTransactionResultWithMetadata, BanksClientError> {

        let bh = environment_context.get_new_latest_blockhash().await?;
        // Create the transaction
        let tx = Transaction::new_signed_with_payer(
            instructions,
            payer,
            signing_keypairs,
            bh);

        // Use banks to submit the tx
        environment_context.banks_client.process_transaction_with_metadata(tx).await
    }

    pub async fn get_account(environment_context: &mut ProgramTestContext, account_pubkey: Pubkey) -> Result<Option<Account>, BanksClientError> {
        let account = environment_context.banks_client.get_account(account_pubkey).await?;

        Ok(account)
    }

    pub fn deserialize_account_data<T: AccountDeserialize>(data: &[u8]) -> Result<T, ProgramError> {
        let result = T::try_deserialize(&mut data.as_ref())?;

        Ok(result)
    }


    pub async fn get_account_data_size(
        environment_context: &mut ProgramTestContext,
        account_pubkey: Pubkey,
    ) -> Result<u64, BanksClientError> {
        // Read the raw account data
        let account = environment_context.banks_client.get_account(account_pubkey).await?.unwrap();

        Ok(account.data.len() as u64)
    }
}