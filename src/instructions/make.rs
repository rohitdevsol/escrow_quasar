use quasar_lang::prelude::*;
use quasar_spl::{ Mint, Token, TokenCpi };
use crate::{ event::MakeEvent, state::Escrow };

#[derive(Accounts)]
pub struct Make<'info> {
    // The user creating the escrow. Signs the tx and pays for the acc creation
    pub maker: &'info mut Signer,

    // created via init with pda seeds. each maker gets one active escrow
    #[account(init, payer = maker, seeds = [b"escrow", maker], bump)]
    pub escrow: &'info mut Account<Escrow>,

    // two token mints. Account<Mint> validates that these are SPL mint accounts
    pub mint_a: &'info Account<Mint>,
    pub mint_b: &'info Account<Mint>,

    // Maker's existing token account for mint A (the deposited tokens)
    pub maker_token_account_for_mint_a: &'info mut Account<Token>,

    // Maker's token account for mint B . created if it does not exist; validates if it does
    #[account(init_if_needed, payer = maker, token::mint = mint_b, token::authority = maker)]
    pub maker_token_account_for_mint_b: &'info mut Account<Token>,

    // vault token account holding escrowed tokens.
    // token::authority=escrow means only the escrow pda can move tokens out via signed CPI
    #[account(init_if_needed, payer = maker, token::mint = mint_a, token::authority = escrow)]
    pub vault_token_account_for_mint_a: &'info mut Account<Token>,

    // required for token creation and operations
    // pub rent: &'info Sysvar<Rent>,
    pub token_program: &'info Program<Token>,
    pub system_program: &'info Program<System>,
}

impl<'info> Make<'info> {
    pub fn make_escrow(&mut self, receive: u64, bumps: &MakeBumps) -> Result<(), ProgramError> {
        self.escrow.set_inner(
            *self.maker.address(),
            *self.mint_a.address(),
            *self.mint_b.address(),
            *self.maker_token_account_for_mint_b.address(),
            receive,
            bumps.escrow
        );
        Ok(())
    }

    pub fn emit_event(&self, deposit: u64, receive: u64) -> Result<(), ProgramError> {
        emit!(MakeEvent {
            escrow: *self.escrow.address(),
            maker: *self.maker.address(),
            mint_a: *self.mint_a.address(),
            mint_b: *self.mint_b.address(),
            deposit,
            receive,
        });
        Ok(())
    }

    pub fn deposit_tokens(&mut self, amount: u64) -> Result<(), ProgramError> {
        self.token_program
            .transfer(
                self.maker_token_account_for_mint_a,
                self.vault_token_account_for_mint_a,
                self.maker,
                amount
            )
            .invoke()
    }
}
