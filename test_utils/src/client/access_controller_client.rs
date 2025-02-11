use access_controller::accounts::{AddAccess, Initialize, RemoveAccess};
use access_controller::instruction::AddAccess as AddAccessParams;
use access_controller::instruction::Initialize as InitializeParams;
use access_controller::instruction::RemoveAccess as RemoveAccessParams;
use anchor_lang::prelude::{ProgramError, Pubkey};
use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::{AccountDeserialize, InstructionData, ToAccountMetas};
use solana_program_test::{BanksClientError, BanksTransactionResultWithMetadata, ProgramTestContext};
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use access_controller::AccessController;
use crate::environment_context_operations::EnvironmentContextOperations;

pub struct AccessControllerClient {
    program_id: Pubkey,
    data_account: Pubkey,
}

impl AccessControllerClient {
    pub fn new(program_id: Pubkey, data_account: Pubkey) -> Self {
        Self {
            program_id,
            data_account,
        }
    }

    pub fn data_account_override(&mut self, data_account: Pubkey) {
        self.data_account = data_account;
    }

    pub async fn add_access(&self,
                            context: &mut ProgramTestContext,
                            owner: &Keypair,
                            user: Pubkey) -> Result<BanksTransactionResultWithMetadata, BanksClientError> {
        let data = AddAccessParams {};

        let add_access_context = AddAccess {
            state: self.data_account,
            owner: owner.pubkey(),
            address: user,
        };

        let instruction = Instruction {
            program_id: self.program_id,
            accounts: add_access_context.to_account_metas(None),
            data: data.data(),
        };

        EnvironmentContextOperations::send_transaction(context, &[instruction], Some(&owner.pubkey()), &[owner]).await
    }

    pub async fn has_access(&self,
                            context: &mut ProgramTestContext,
                            user: &Pubkey) -> bool {
        // Read the access controller account
        let access_controller = self.read_access_controller_account(context)
            .await
            .expect("Failed to read access controller account");

        // Check if user is in the access list using binary search
        access_controller.access_list.binary_search(user).is_ok()
    }

    pub async fn remove_access(
        &self,
        context: &mut ProgramTestContext,
        owner: &Keypair,
        user: Pubkey,
    ) -> Result<BanksTransactionResultWithMetadata, BanksClientError> {
        let data = RemoveAccessParams {};
        
        let remove_access_context = RemoveAccess {
            state: self.data_account,
            owner: owner.pubkey(),
            address: user,
        };

        let instruction = Instruction {
            program_id: self.program_id,
            accounts: remove_access_context.to_account_metas(None),
            data: data.data(),
        };

        EnvironmentContextOperations::send_transaction(
            context,
            &[instruction],
            Some(&owner.pubkey()),
            &[owner],
        ).await
    }

    pub async fn initialize(&self,
                            context: &mut ProgramTestContext,
                            user: &Keypair) -> Result<BanksTransactionResultWithMetadata, BanksClientError> {
        let data = InitializeParams {};

        let initialize_context = Initialize {
            state: self.data_account,
            owner: user.pubkey(),
        };

        let instruction = Instruction {
            program_id: self.program_id,
            accounts: initialize_context.to_account_metas(None),
            data: data.data(),
        };

        EnvironmentContextOperations::send_transaction(context, &[instruction], Some(&user.pubkey()), &[user]).await
    }


    pub async fn read_access_controller_account(
        &self,
        context: &mut ProgramTestContext,
    ) -> Result<access_controller::AccessController, ProgramError> {
        let account = EnvironmentContextOperations::get_account(context, self.data_account)
            .await
            .unwrap()
            .unwrap();

        // Verify account size
        if account.data.len() < 8 + std::mem::size_of::<AccessController>() {
            return Err(ProgramError::AccountDataTooSmall);
        }

        let access_controller: AccessController =
            AccessController::try_deserialize(&mut &account.data[..])
                .map_err(|_| ProgramError::InvalidAccountData)?;

        Ok(access_controller)
    }
}
