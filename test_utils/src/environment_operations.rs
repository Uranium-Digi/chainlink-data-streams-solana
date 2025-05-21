use anchor_lang::prelude::Pubkey;
use solana_program::rent::Rent;
use solana_program_test::ProgramTest;
use solana_sdk::account::Account;
use solana_sdk::signature::{Keypair, Signer};

pub struct TestEnvironmentHelper {}

impl TestEnvironmentHelper {
    pub fn add_account(environment: &mut ProgramTest, user: &Pubkey, lamports: u64) {
        environment.add_account(
            *user,
            Account {
                lamports,
                ..Account::default()
            },
        );
    }

    pub fn add_upgradable_program(
        environment: &mut ProgramTest,
        program_name: &str,
        program_id: Pubkey,
        program_authority: Option<Pubkey>,
    ) {
        let (program_data_address, _) = Pubkey::find_program_address(
            &[program_id.as_ref()],
            &solana_program::bpf_loader_upgradeable::id(),
        );
        panic!("Not supported yet");
        /*
        environment.add_bpf_program_with_program_data(
            program_name,
            program_id,
            program_authority,
            program_data_address,
            None,
        );
        */
    }

    pub fn add_program(environment: &mut ProgramTest, program_name: &'static str, program_id: Pubkey) {
        environment.add_program(program_name, program_id, None);
    }

    pub fn add_zero_copy_account(
        environment: &mut ProgramTest,
        account_size_bytes_excluding_discriminator: usize,
        account_pubkey: Pubkey,
        owner: Pubkey,
        discriminator: Option<[u8; 8]>,
    ) {
        let account_size = 8 + account_size_bytes_excluding_discriminator;
        let mut data = vec![0; account_size];
        if let Some(disc) = discriminator {
            data[..8].copy_from_slice(&disc);
        }
        environment.add_account(
            account_pubkey,
            Account {
                lamports: Rent::default().minimum_balance(account_size),
                data,
                owner,
                executable: false,
                rent_epoch: 0,
            },
        );
    }
}

pub struct TestEnvironment {
    pub environment: ProgramTest,
    pub user: Keypair,
}

impl TestEnvironment {
    pub fn new(program_name: &'static str, program_id: Pubkey, compute_max_units: Option<u64>) -> Self {
        // Set up the environment
        let mut environment = ProgramTest::default();

        // Set up the contracts
        TestEnvironmentHelper::add_program(&mut environment, program_name, program_id);

        // Add a default user to the environment who will be the payer of the deployment
        let user = Keypair::new();
        TestEnvironmentHelper::add_account(&mut environment, &user.pubkey(), 1_000_000_000_000);

        // Set the compute max units if provided
        if let Some(max_units) = compute_max_units {
            environment.set_compute_max_units(max_units);
        }

        Self { environment, user }
    }
}
