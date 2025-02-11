use crate::environment_operations::{TestEnvironment, TestEnvironmentHelper};
use solana_program_test::ProgramTestContext;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;

pub struct BaseTestSetup {
    pub environment_context: ProgramTestContext,
    pub user: Keypair,
}

pub struct BaseTestSetupBuilder {
    program_name: String,
    program_id: Pubkey,
    account_size: usize,
    additional_users: Vec<Pubkey>,
    additional_programs: Vec<(String, Pubkey)>,
    additional_zero_copy_accounts: Vec<(Pubkey, Pubkey, usize, Option<[u8; 8]>)>,
    compute_max_units: Option<u64>,
}

impl BaseTestSetupBuilder {
    pub fn new() -> Self {
        Self {
            program_name: "default_program".to_string(),
            program_id: Pubkey::default(),
            account_size: 0,
            additional_users: Vec::new(),
            additional_programs: Vec::new(),
            additional_zero_copy_accounts: Vec::new(),
            compute_max_units: None,
        }
    }

    pub fn program_name(mut self, name: &str) -> Self {
        self.program_name = name.to_string();
        self
    }

    pub fn program_id(mut self, id: Pubkey) -> Self {
        self.program_id = id;
        self
    }

    pub fn account_size(mut self, size: usize) -> Self {
        self.account_size = size;
        self
    }

    pub fn add_user(mut self, user: Pubkey) -> Self {
        self.additional_users.push(user);
        self
    }

    pub fn add_program(mut self, name: &str, program_id: Pubkey) -> Self {
        self.additional_programs.push((name.to_string(), program_id));
        self
    }

    pub fn add_zero_copy_account(mut self, program_id: Pubkey, account: Pubkey, account_size: usize, discriminator: Option<[u8; 8]>) -> Self {
        self.additional_zero_copy_accounts.push((program_id, account, account_size, discriminator));
        self
    }

    pub fn with_compute_max_units(mut self, max_units: u64) -> Self {
        self.compute_max_units = Some(max_units);
        self
    }

    pub async fn build(self) -> BaseTestSetup {

        // 1. Setup the test environment
        let TestEnvironment { mut environment, user } = TestEnvironment::new(
            &self.program_name,
            self.program_id,
            self.compute_max_units,
        );

        // 2. Add the additional users
        for additional_user in &self.additional_users {
            TestEnvironmentHelper::add_account(
                &mut environment,
                additional_user,
                1_000_000_000_000,
            );
        }

        // Will override the program with the same name.
        // Confirm with test log "Overriding account at <program id>"
        TestEnvironmentHelper::add_upgradable_program(
            &mut environment,
            &self.program_name,
            self.program_id,
            Some(user.pubkey()),
        );

        // 3. Add the additional programs
        for (program_name, program_id) in &self.additional_programs {
            TestEnvironmentHelper::add_program(&mut environment, program_name, *program_id);
        }

        // 4. Add the additional zero-copy accounts
        for (program_id, account, account_size, discriminator) in &self.additional_zero_copy_accounts {
            TestEnvironmentHelper::add_zero_copy_account(
                &mut environment,
                *account_size,
                *account,
                *program_id,
                *discriminator,
            );
        }

        // 5. Initialize EnvironmentContextOperations
        let environment_context = environment.start_with_context().await;

        BaseTestSetup {
            environment_context,
            user
        }
    }
}
