use crate::environment_context_operations::EnvironmentContextOperations;
use anchor_lang::prelude::{ProgramError, Pubkey};
use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::{AccountDeserialize, InstructionData, ToAccountMetas};
use solana_program::system_program;
use solana_program_test::{
    BanksClientError, BanksTransactionResultWithMetadata, ProgramTestContext,
};
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use verifier::accounts::InitializeAccountDataContext;
use verifier::accounts::{
    AcceptOwnershipContext, InitializeContext, ReallocContext, SetAccessControllerContext,
    TransferOwnershipContext, UpdateConfigContext, VerifyContext,
};
use verifier::instruction::AcceptOwnership as AcceptOwnershipParams;
use verifier::instruction::Initialize as InitializeParams;
use verifier::instruction::InitializeAccountData;
use verifier::instruction::ReallocAccount as ReallocParams;
use verifier::instruction::RemoveLatestConfig as RemoveLatestConfigParams;
use verifier::instruction::SetAccessController as SetAccessControllerParams;
use verifier::instruction::SetConfig as SetConfigParams;
use verifier::instruction::SetConfigActive as SetConfigActiveParams;
use verifier::instruction::SetConfigWithActivationTime as SetConfigWithActivationTimeParams;
use verifier::instruction::TransferOwnership as TransferOwnershipParams;
use verifier::instruction::Verify as VerifyParams;
use verifier::state::VerifierAccount;
use verifier::util::Compressor;

// Verifier struct using ContractOperations
// This is a client wrapper to abstract interacting with the Verifier program
pub struct VerifierClient {
    program_id: Pubkey,
    pub access_controller_data_account: Option<Pubkey>,
    pub data_account: Pubkey,
}

impl VerifierClient {
    pub fn new(program_id: Pubkey, access_controller_data_account: Option<Pubkey>) -> Self {
        let (data_account, _bump) = Pubkey::find_program_address(&[b"verifier"], &program_id);

        Self {
            program_id,
            access_controller_data_account,
            data_account,
        }
    }

    pub async fn initialize(
        &self,
        context: &mut ProgramTestContext,
        user: &Keypair,
    ) -> Result<BanksTransactionResultWithMetadata, BanksClientError> {
        let data = InitializeParams {};

        let initialize_context = InitializeContext {
            verifier_account: self.data_account,
            owner: user.pubkey(),
            program: self.program_id,
            program_data: self.get_program_data_address(),
            system_program: system_program::ID,
        };

        let instruction = Instruction {
            program_id: self.program_id,
            accounts: initialize_context.to_account_metas(None),
            data: data.data(),
        };

        EnvironmentContextOperations::send_transaction(
            context,
            &[instruction],
            Some(&user.pubkey()),
            &[user],
        )
        .await
    }

    pub async fn realloc(
        &self,
        context: &mut ProgramTestContext,
        user: &Keypair,
        len: usize,
    ) -> Result<BanksTransactionResultWithMetadata, BanksClientError> {
        let _len = len as u32;
        let params = ReallocParams { _len };

        let realloc_context = ReallocContext {
            verifier_account: self.data_account,
            owner: user.pubkey(),
            system_program: system_program::ID,
            program: self.program_id,
            program_data: self.get_program_data_address(),
        };

        let instruction = Instruction {
            program_id: self.program_id,
            accounts: realloc_context.to_account_metas(None),
            data: params.data(),
        };

        EnvironmentContextOperations::send_transaction(
            context,
            &[instruction],
            Some(&user.pubkey()),
            &[user],
        )
        .await
    }

    pub async fn init_data(
        &self,
        context: &mut ProgramTestContext,
        user: &Keypair,
    ) -> Result<BanksTransactionResultWithMetadata, BanksClientError> {
        let data = InitializeAccountData {};

        let initialize_context = InitializeAccountDataContext {
            verifier_account: self.data_account,
            owner: user.pubkey(),
            access_controller: self.access_controller_data_account,
            system_program: system_program::ID,
            program: self.program_id,
            program_data: self.get_program_data_address(),
        };

        let instruction = Instruction {
            program_id: self.program_id,
            accounts: initialize_context.to_account_metas(None),
            data: data.data(),
        };

        EnvironmentContextOperations::send_transaction(
            context,
            &[instruction],
            Some(&user.pubkey()),
            &[user],
        )
        .await
    }

    /// This will reallocate the account to the full size required for the verifier account
    /// using multiple realloc transaction calls
    pub async fn realloc_full_size(
        &self,
        context: &mut ProgramTestContext,
        user: &Keypair,
    ) -> Result<BanksTransactionResultWithMetadata, BanksClientError> {
        // 8 bytes for account discriminator
        let target_size = 8 + std::mem::size_of::<VerifierAccount>();
        let mut current_size = VerifierAccount::INIT_SPACE;
        const REALLOC_INCREMENT: usize = 10 * 1024;

        // Perform reallocation in increments
        while current_size < target_size {
            current_size = std::cmp::min(current_size + REALLOC_INCREMENT, target_size);
            let res = self.realloc(context, user, current_size).await?;
            if res.result.is_err() {
                return Ok(res);
            }
        }

        Ok(BanksTransactionResultWithMetadata {
            result: Ok(()),
            metadata: None,
        })
    }

    /// This function initializes the verifier account, reallocates the account to full size, and
    /// initializes the account data.
    pub async fn initialize_realloc_init_data(
        &self,
        context: &mut ProgramTestContext,
        user: &Keypair,
    ) -> Result<BanksTransactionResultWithMetadata, BanksClientError> {
        let _result = self.initialize(context, user).await?;

        let _result = self.realloc_full_size(context, user).await?;

        let result = self.init_data(context, user).await?;

        Ok(result)
    }

    pub async fn set_access_controller(
        &self,
        context: &mut ProgramTestContext,
        user: &Keypair,
        access_controller_data_account: Option<Pubkey>,
    ) -> Result<BanksTransactionResultWithMetadata, BanksClientError> {
        let data = SetAccessControllerParams {};

        let set_access_controller_context = SetAccessControllerContext {
            verifier_account: self.data_account,
            owner: user.pubkey(),
            access_controller: access_controller_data_account,
        };

        let instruction = Instruction {
            program_id: self.program_id,
            accounts: set_access_controller_context.to_account_metas(None),
            data: data.data(),
        };

        EnvironmentContextOperations::send_transaction(
            context,
            &[instruction],
            Some(&user.pubkey()),
            &[user],
        )
        .await
    }

    pub async fn verify(
        &self,
        context: &mut ProgramTestContext,
        user: &Keypair,
        signed_report: Vec<u8>,
        override_config_account: Option<Pubkey>,
    ) -> Result<BanksTransactionResultWithMetadata, BanksClientError> {
        let permissioned_context = VerifyContext {
            verifier_account: self.data_account,
            user: user.pubkey(),
            access_controller: self.access_controller_data_account.unwrap(),
            config_account: override_config_account
                .unwrap_or(self.compute_report_config_pda(&signed_report)),
        };

        let data = VerifyParams { signed_report };

        let instruction = Instruction {
            program_id: self.program_id,
            accounts: permissioned_context.to_account_metas(None),
            data: data.data(),
        };

        EnvironmentContextOperations::send_transaction(
            context,
            &[instruction],
            Some(&user.pubkey()),
            &[user],
        )
        .await
    }

    pub async fn set_config_with_activation_time(
        &self,
        context: &mut ProgramTestContext,
        user: &Keypair,
        signers: Vec<[u8; 20]>,
        f: u8,
        activation_time: u32,
    ) -> Result<BanksTransactionResultWithMetadata, BanksClientError> {
        let data = SetConfigWithActivationTimeParams {
            signers,
            f,
            activation_time,
        };

        let owner_context = UpdateConfigContext {
            verifier_account: self.data_account,
            owner: user.pubkey(),
        };

        let instruction = Instruction {
            program_id: self.program_id,
            accounts: owner_context.to_account_metas(None),
            data: data.data(),
        };

        EnvironmentContextOperations::send_transaction(
            context,
            &[instruction],
            Some(&user.pubkey()),
            &[user],
        )
        .await
    }

    pub async fn set_config(
        &self,
        context: &mut ProgramTestContext,
        user: &Keypair,
        signers: Vec<[u8; 20]>,
        f: u8,
    ) -> Result<BanksTransactionResultWithMetadata, BanksClientError> {
        let data = SetConfigParams { signers, f };

        let owner_context = UpdateConfigContext {
            verifier_account: self.data_account,
            owner: user.pubkey(),
        };

        let instruction = Instruction {
            program_id: self.program_id,
            accounts: owner_context.to_account_metas(None),
            data: data.data(),
        };

        EnvironmentContextOperations::send_transaction(
            context,
            &[instruction],
            Some(&user.pubkey()),
            &[user],
        )
        .await
    }

    pub async fn set_config_active(
        &self,
        context: &mut ProgramTestContext,
        user: &Keypair,
        don_config_index: u64,
        is_active: bool,
    ) -> Result<BanksTransactionResultWithMetadata, BanksClientError> {
        let data = SetConfigActiveParams {
            don_config_index,
            is_active: u8::from(is_active),
        };

        let owner_context = UpdateConfigContext {
            verifier_account: self.data_account,
            owner: user.pubkey(),
        };

        let instruction = Instruction {
            program_id: self.program_id,
            accounts: owner_context.to_account_metas(None),
            data: data.data(),
        };

        EnvironmentContextOperations::send_transaction(
            context,
            &[instruction],
            Some(&user.pubkey()),
            &[user],
        )
        .await
    }

    pub async fn remove_latest_config(
        &self,
        context: &mut ProgramTestContext,
        user: &Keypair,
    ) -> Result<BanksTransactionResultWithMetadata, BanksClientError> {
        let data = RemoveLatestConfigParams {};

        let owner_context = UpdateConfigContext {
            verifier_account: self.data_account,
            owner: user.pubkey(),
        };

        let instruction = Instruction {
            program_id: self.program_id,
            accounts: owner_context.to_account_metas(None),
            data: data.data(),
        };

        EnvironmentContextOperations::send_transaction(
            context,
            &[instruction],
            Some(&user.pubkey()),
            &[user],
        )
        .await
    }

    pub async fn transfer_ownership(
        &self,
        context: &mut ProgramTestContext,
        user: &Keypair,
        proposed_owner: Pubkey,
    ) -> Result<BanksTransactionResultWithMetadata, BanksClientError> {
        let data = TransferOwnershipParams { proposed_owner };

        let transfer_ownership_context = TransferOwnershipContext {
            verifier_account: self.data_account,
            owner: user.pubkey(),
        };

        let instruction = Instruction {
            program_id: self.program_id,
            accounts: transfer_ownership_context.to_account_metas(None),
            data: data.data(),
        };

        EnvironmentContextOperations::send_transaction(
            context,
            &[instruction],
            Some(&user.pubkey()),
            &[user],
        )
        .await
    }

    pub async fn accept_ownership(
        &self,
        context: &mut ProgramTestContext,
        user: &Keypair,
    ) -> Result<BanksTransactionResultWithMetadata, BanksClientError> {
        let data = AcceptOwnershipParams {};

        let accept_ownership_context = AcceptOwnershipContext {
            verifier_account: self.data_account,
            owner: user.pubkey(),
        };

        let instruction = Instruction {
            program_id: self.program_id,
            accounts: accept_ownership_context.to_account_metas(None),
            data: data.data(),
        };

        EnvironmentContextOperations::send_transaction(
            context,
            &[instruction],
            Some(&user.pubkey()),
            &[user],
        )
        .await
    }

    pub async fn read_verifier_account(
        &self,
        context: &mut ProgramTestContext,
    ) -> Result<VerifierAccount, ProgramError> {
        let account = EnvironmentContextOperations::get_account(context, self.data_account)
            .await
            .unwrap()
            .unwrap();
        
        // Verify account size
        if account.data.len() < 8 + std::mem::size_of::<VerifierAccount>() {
            return Err(ProgramError::AccountDataTooSmall);
        }

        let verifier_account: VerifierAccount =
            VerifierAccount::try_deserialize(&mut &account.data[..])
                .map_err(|_| ProgramError::InvalidAccountData)?;

        Ok(verifier_account)
    }

    pub fn access_controller_data_account_override(
        &mut self,
        access_controller_data_account: Option<Pubkey>,
    ) {
        self.access_controller_data_account = access_controller_data_account;
    }

    pub fn data_account_override(&mut self, data_account: Pubkey) {
        self.data_account = data_account;
    }

    fn get_program_data_address(&self) -> Pubkey {
        let (program_data_address, _) = Pubkey::find_program_address(
            &[self.program_id.as_ref()],
            &solana_program::bpf_loader_upgradeable::id(),
        );
        program_data_address
    }

    pub fn compute_report_config_pda(&self, report: &[u8]) -> Pubkey {
        let r = Compressor::decompress(report);
        let seed = &r[..32];
        let (program_data_address, _) = Pubkey::find_program_address(&[seed], &self.program_id);
        program_data_address
    }
}
