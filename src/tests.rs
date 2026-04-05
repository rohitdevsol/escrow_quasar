extern crate std;

use escrow_quasar_client::MakeInstruction;
use quasar_svm::{ Account, Instruction, Pubkey, QuasarSvm, token::{ Mint } };
use solana_address::Address;
use solana_keypair::{ Keypair, Signer };
use solana_program_option::COption;
use spl_token::state::Account as SplTokenAccount;
use solana_program_pack::Pack;

struct State {
    maker: Keypair,
    mint_a: Pubkey,
    mint_b: Pubkey,
    escrow: Address,
    maker_ata_a: Address,
    maker_ata_b: Address,
    vault_ata_a: Address,
}

const MINT_SPEC: Mint = Mint {
    is_initialized: true,
    freeze_authority: COption::None,
    decimals: 6,
    mint_authority: COption::None,
    supply: 100_000_000_000,
};
fn setup() -> (QuasarSvm, State) {
    let elf = include_bytes!("../target/deploy/escrow_quasar.so");
    let mut svm = QuasarSvm::new().with_program(&Pubkey::from(crate::ID), elf);

    // return a resuable state from here

    let maker = Keypair::new();
    let mint_a = Pubkey::new_unique();
    let mint_b = Pubkey::new_unique();

    let (escrow, _) = Address::find_program_address(
        &[b"escrow", maker.pubkey().as_ref()],
        &crate::ID
    );

    // maker - already funded account
    svm.set_account(Account {
        address: maker.pubkey(),
        lamports: 20_000_000_000,
        data: vec![],
        owner: quasar_svm::system_program::ID,
        executable: false,
    });

    // 2 mints for initial state
    svm.set_account(quasar_svm::token::create_keyed_mint_account(&mint_a, &MINT_SPEC));
    svm.set_account(quasar_svm::token::create_keyed_mint_account(&mint_b, &MINT_SPEC));

    // derive ATAs for maker
    let (maker_ata_a, _) = Pubkey::find_program_address(
        &[maker.pubkey().as_ref(), quasar_svm::SPL_TOKEN_PROGRAM_ID.as_ref(), mint_a.as_ref()],
        &quasar_svm::SPL_ASSOCIATED_TOKEN_PROGRAM_ID
    );

    let (maker_ata_b, _) = Pubkey::find_program_address(
        &[maker.pubkey().as_ref(), quasar_svm::SPL_TOKEN_PROGRAM_ID.as_ref(), mint_b.as_ref()],
        &quasar_svm::SPL_ASSOCIATED_TOKEN_PROGRAM_ID
    );

    // maker ATA for mint_a - has tokens to deposit
    svm.set_account(
        quasar_svm::token::create_keyed_associated_token_account(
            &maker.pubkey(),
            &mint_a,
            50_000_000_000
        )
    );

    // maker ATA for mint_b - receives tokens, starts empty
    svm.set_account(
        quasar_svm::token::create_keyed_associated_token_account(&maker.pubkey(), &mint_b, 0)
    );

    let (vault_ata_a, _) = Pubkey::find_program_address(
        &[escrow.as_ref(), quasar_svm::SPL_TOKEN_PROGRAM_ID.as_ref(), mint_a.as_ref()],
        &quasar_svm::SPL_ASSOCIATED_TOKEN_PROGRAM_ID
    );

    // vault ATA — owned by escrow PDA, starts empty
    svm.set_account(
        quasar_svm::token::create_keyed_associated_token_account(
            &Pubkey::from(escrow.to_bytes()),
            &mint_a,
            0
        )
    );

    // escrow PDA — uninitialized system account, Make will init it
    svm.set_account(Account {
        address: Pubkey::from(escrow.to_bytes()),
        lamports: 0,
        data: vec![],
        owner: quasar_svm::system_program::ID,
        executable: false,
    });

    let state = State {
        maker,
        mint_a,
        mint_b,
        escrow,
        maker_ata_a,
        maker_ata_b,
        vault_ata_a,
    };
    (svm, state)
}

#[test]
pub fn make_escrow() {
    let (mut svm, state) = setup();

    let State { maker, mint_a, mint_b, escrow, maker_ata_a, maker_ata_b, vault_ata_a } = state;

    let maker_ata_a_before = svm.get_account(&maker_ata_a).unwrap();

    let make_ix: Instruction = (MakeInstruction {
        deposit: 5_000_000_000,
        escrow,
        maker: maker.pubkey(),
        maker_token_account_for_mint_a: maker_ata_a,
        maker_token_account_for_mint_b: maker_ata_b,
        mint_a,
        mint_b,
        receive: 50,
        system_program: Address::from(quasar_svm::system_program::ID.to_bytes()),
        token_program: Address::from(quasar_svm::SPL_TOKEN_PROGRAM_ID.to_bytes()),
        vault_token_account_for_mint_a: vault_ata_a,
    }).into();

    let make_result = svm.process_instruction(&make_ix, &[]);
    make_result.assert_success();

    // capture AFTER state
    let maker_ata_a_after = make_result.account(&maker_ata_a).unwrap();

    let before_data = maker_ata_a_before.data.as_slice();
    let before_token = SplTokenAccount::unpack(before_data).unwrap();

    let after_data = maker_ata_a_after.data.as_slice();
    let after_token = SplTokenAccount::unpack(after_data).unwrap();

    eprintln!("MAKER ATA A BEFORE amount: {}", before_token.amount);
    eprintln!("MAKER ATA A AFTER amount:  {}", after_token.amount);
}
