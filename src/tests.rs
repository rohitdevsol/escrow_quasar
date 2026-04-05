extern crate std;

use escrow_quasar_client::{ MakeInstruction, RefundInstruction, TakeInstruction };
use quasar_svm::{ Account, ExecutionResult, Instruction, Pubkey, QuasarSvm, token::Mint };
use solana_address::Address;
use solana_keypair::{ Keypair, Signer };
use solana_program_option::COption;
use spl_token::state::Account as SplTokenAccount;
use solana_program_pack::Pack;

struct State {
    maker: Keypair,
    taker: Keypair,
    mint_a: Pubkey,
    mint_b: Pubkey,
    escrow: Address,
    maker_ata_a: Address,
    maker_ata_b: Address,
    taker_ata_b: Address,
    taker_ata_a: Address,
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
    let taker = Keypair::new();
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

    // taker is also funfed account
    svm.set_account(Account {
        address: taker.pubkey(),
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

    let (taker_ata_a, _) = Pubkey::find_program_address(
        &[taker.pubkey().as_ref(), quasar_svm::SPL_TOKEN_PROGRAM_ID.as_ref(), mint_a.as_ref()],
        &quasar_svm::SPL_ASSOCIATED_TOKEN_PROGRAM_ID
    );

    let (taker_ata_b, _) = Pubkey::find_program_address(
        &[taker.pubkey().as_ref(), quasar_svm::SPL_TOKEN_PROGRAM_ID.as_ref(), mint_b.as_ref()],
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

    // taker ATA on mint b. to give out the tokens for exhanging
    svm.set_account(
        quasar_svm::token::create_keyed_associated_token_account(
            &taker.pubkey(),
            &mint_b,
            50_000_000_000
        )
    );

    // maker ATA for mint_b - receives tokens, starts empty
    svm.set_account(
        quasar_svm::token::create_keyed_associated_token_account(&maker.pubkey(), &mint_b, 0)
    );

    // taker ATA for mint_a - receives tokens, starts empty
    svm.set_account(
        quasar_svm::token::create_keyed_associated_token_account(&taker.pubkey(), &mint_a, 0)
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
        taker,
        mint_a,
        mint_b,
        escrow,
        maker_ata_a,
        maker_ata_b,
        taker_ata_a,
        taker_ata_b,
        vault_ata_a,
    };
    (svm, state)
}

fn run_make(svm: &mut QuasarSvm, state: &State) -> ExecutionResult {
    let make_ix: Instruction = (MakeInstruction {
        deposit: 5_000_000_000,
        escrow: state.escrow,
        maker: state.maker.pubkey(),
        maker_token_account_for_mint_a: state.maker_ata_a,
        maker_token_account_for_mint_b: state.maker_ata_b,
        mint_a: state.mint_a,
        mint_b: state.mint_b,
        receive: 50,
        system_program: Address::from(quasar_svm::system_program::ID.to_bytes()),
        token_program: Address::from(quasar_svm::SPL_TOKEN_PROGRAM_ID.to_bytes()),
        vault_token_account_for_mint_a: state.vault_ata_a,
    }).into();

    let result = svm.process_instruction(&make_ix, &[]);
    result
}

fn run_take(svm: &mut QuasarSvm, state: &State) -> ExecutionResult {
    let take_ix: Instruction = (TakeInstruction {
        escrow: state.escrow,
        maker: state.maker.pubkey(),
        maker_token_account_for_mint_b: state.maker_ata_b,
        mint_a: state.mint_a,
        mint_b: state.mint_b,
        taker: state.taker.pubkey(),
        taker_token_account_for_mint_b: state.taker_ata_b,
        taker_token_account_for_mint_a: state.taker_ata_a,
        system_program: Address::from(quasar_svm::system_program::ID.to_bytes()),
        token_program: Address::from(quasar_svm::SPL_TOKEN_PROGRAM_ID.to_bytes()),
        vault_token_account_for_mint_a: state.vault_ata_a,
    }).into();
    let result = svm.process_instruction(&take_ix, &[]);
    result
}

fn run_refund(svm: &mut QuasarSvm, state: &State) -> ExecutionResult {
    let refund_ix: Instruction = (RefundInstruction {
        escrow: state.escrow,
        maker: state.maker.pubkey(),
        maker_token_account_for_mint_a: state.maker_ata_a,
        mint_a: state.mint_a,
        system_program: Address::from(quasar_svm::system_program::ID.to_bytes()),
        token_program: Address::from(quasar_svm::SPL_TOKEN_PROGRAM_ID.to_bytes()),
        vault_token_account_for_mint_a: state.vault_ata_a,
    }).into();

    let result = svm.process_instruction(&refund_ix, &[]);
    result
}

#[test]
pub fn make_escrow() {
    let (mut svm, state) = setup();
    let result = run_make(&mut svm, &state);
    result.assert_success();

    let vault = SplTokenAccount::unpack(
        result.account(&state.vault_ata_a).unwrap().data.as_slice()
    ).unwrap();
    assert_eq!(vault.amount, 5_000_000_000);

    let maker_ata_a = SplTokenAccount::unpack(
        result.account(&state.maker_ata_a).unwrap().data.as_slice()
    ).unwrap();
    assert_eq!(maker_ata_a.amount, 45_000_000_000);
}

#[test]
fn take_escrow() {
    let (mut svm, state) = setup();
    run_make(&mut svm, &state).assert_success();
    let result = run_take(&mut svm, &state);
    result.assert_success();

    assert!(result.account(&state.vault_ata_a).unwrap().data.is_empty());

    let taker_ata_a = SplTokenAccount::unpack(
        result.account(&state.taker_ata_a).unwrap().data.as_slice()
    ).unwrap();
    assert_eq!(taker_ata_a.amount, 5_000_000_000);

    let maker_ata_b = SplTokenAccount::unpack(
        result.account(&state.maker_ata_b).unwrap().data.as_slice()
    ).unwrap();
    assert_eq!(maker_ata_b.amount, 50);

    let taker_ata_b = SplTokenAccount::unpack(
        result.account(&state.taker_ata_b).unwrap().data.as_slice()
    ).unwrap();
    assert_eq!(taker_ata_b.amount, 49_999_999_950);
}

#[test]
fn refund_escrow() {
    let (mut svm, state) = setup();

    run_make(&mut svm, &state).assert_success();
    let result = run_refund(&mut svm, &state);
    result.assert_success();

    let maker_ata_a = SplTokenAccount::unpack(
        result.account(&state.maker_ata_a).unwrap().data.as_slice()
    ).unwrap();
    assert_eq!(maker_ata_a.amount, 50_000_000_000);

    assert!(result.account(&state.vault_ata_a).unwrap().data.is_empty());
}

#[test]
fn refund_escrow_should_fail() {
    let (mut svm, state) = setup();

    run_make(&mut svm, &state).assert_success();
    run_take(&mut svm, &state).assert_success();
    run_refund(&mut svm, &state).assert_error(
        quasar_svm::ProgramError::Runtime(String::from("IllegalOwner"))
    );
}
