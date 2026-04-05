extern crate std;

use escrow_quasar_client::MakeInstruction;
use quasar_svm::{
    Account,
    Instruction,
    Pubkey,
    QuasarSvm,
    SPL_ASSOCIATED_TOKEN_PROGRAM_ID,
    SPL_TOKEN_PROGRAM_ID,
    token::{ Mint, TokenAccount, create_keyed_mint_account },
};
use solana_address::Address;
use solana_keypair::{ Keypair, Signer };
use solana_program_option::COption;
use spl_token::state::AccountState;
use solana_instruction::{ AccountMeta };

fn setup() -> QuasarSvm {
    let elf = include_bytes!("../target/deploy/escrow_quasar.so");
    QuasarSvm::new().with_program(&Pubkey::from(crate::ID), elf)
}

fn setup_maker() -> [Pubkey; 5] {
    // ek maker hoga
    let maker = Pubkey::new_unique();
    // first mint
    let mint_a = Pubkey::new_unique();
    //second mint
    let mint_b = Pubkey::new_unique();
    // maker token accounts
    let maker_token_account_for_mint_a = Pubkey::new_unique();
    let maker_token_account_for_mint_b = Pubkey::new_unique();
    [maker, mint_a, mint_b, maker_token_account_for_mint_a, maker_token_account_for_mint_b]
}

fn setup_vault() -> Pubkey {
    Pubkey::new_unique()
}

#[test]
pub fn make_escrow() {
    let mut svm = setup();

    let [maker, mint_a, mint_b, maker_token_account_for_mint_a, maker_token_account_for_mint_b] =
        setup_maker();

    let vault = setup_vault();

    // svm.set_account(
    //     quasar_svm::token::create_keyed_token_account(
    //         &maker_token_account_for_mint_b,
    //         &(TokenAccount {
    //             mint: mint_b,
    //             owner: maker,
    //             amount: 0,
    //             state: AccountState::Initialized,
    //             ..Default::default()
    //         })
    //     )
    // );

    let (escrow, _) = Address::find_program_address(&[b"escrow", maker.as_ref()], &crate::ID);

    // svm.set_account(Account {
    //     address: escrow,
    //     lamports: 0,
    //     data: vec![],
    //     owner: crate::ID,
    //     executable: false,
    // });

    let make_ix: Instruction = (MakeInstruction {
        deposit: 5_000_000_000,
        escrow,
        maker,
        maker_token_account_for_mint_a,
        maker_token_account_for_mint_b,
        mint_a,
        mint_b,
        receive: 50,
        system_program: Address::from(quasar_svm::system_program::ID.to_bytes()),
        token_program: Address::from(quasar_svm::SPL_TOKEN_PROGRAM_ID.to_bytes()),
        vault_token_account_for_mint_a: vault,
    }).into();

    svm.set_account(quasar_svm::token::create_keyed_system_account(&maker, 20_000_000_000));

    svm.set_account(Account {
        address: vault,
        lamports: 0,
        data: vec![],
        owner: escrow,
        executable: false,
    });

    svm.set_account(
        quasar_svm::token::create_keyed_mint_account(
            &mint_a,
            &(Mint {
                is_initialized: true,
                freeze_authority: COption::None,
                decimals: 0,
                mint_authority: Some(maker),
                supply: 100_000_000_000,
            })
        )
    );
    svm.set_account(
        quasar_svm::token::create_keyed_mint_account(
            &mint_b,
            &(Mint {
                is_initialized: true,
                freeze_authority: COption::None,
                decimals: 0,
                mint_authority: COption::Some(maker),
                supply: 100_000_000_000,
            })
        )
    );
    svm.set_account(
        quasar_svm::token::create_keyed_token_account(
            &maker_token_account_for_mint_a,
            &(TokenAccount {
                mint: mint_a,
                owner: maker,
                amount: 50_000_000_000,
                state: AccountState::Initialized,
                ..Default::default()
            })
        )
    );

    let make_result = svm.process_instruction(&make_ix, &[]);

    make_result.assert_success();
}
