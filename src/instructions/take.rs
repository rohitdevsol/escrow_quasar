use quasar_lang::prelude::*;
use quasar_spl::{ Mint, Token, TokenCpi };

use crate::{ event::TakeEvent, state::Escrow };

#[derive(Accounts)]
pub struct Take<'info> {
    pub taker: &'info mut Signer,
    #[account(
        has_one = maker, //checks escrow.maker == maker.address()
        has_one = maker_token_account_for_mint_b, // ensures token goes to the right destination
        constraint = escrow.receive > 0, // Escrow must have a non-zero receive amount
        close = taker, // Closes the escrow and sends rent to the taker
        seeds = [b"escrow", maker],
        bump = escrow.bump // uses the stored bump instead of re-deriving, (saves compute units)
    )]
    pub escrow: &'info mut Account<Escrow>,

    pub maker: &'info mut UncheckedAccount,
    pub mint_a: &'info Account<Mint>,
    pub mint_b: &'info Account<Mint>,

    #[account(init_if_needed, payer = taker, token::mint = mint_a, token::authority = taker)]
    pub taker_token_account_for_mint_a: &'info mut Account<Token>,

    pub taker_token_account_for_mint_b: &'info mut Account<Token>,

    #[account(init_if_needed, payer = taker, token::mint = mint_b, token::authority = maker)]
    pub maker_token_account_for_mint_b: &'info mut Account<Token>,

    pub vault_token_account_for_mint_a: &'info mut Account<Token>,

    // pub rent: &'info Sysvar<Rent>,
    pub token_program: &'info Program<Token>,
    pub system_program: &'info Program<System>,
}

impl<'info> Take<'info> {
    pub fn transfer_tokens(&mut self) -> Result<(), ProgramError> {
        self.token_program
            .transfer(
                self.taker_token_account_for_mint_b,
                self.maker_token_account_for_mint_b,
                self.taker,
                self.escrow.receive
            )
            .invoke()
    }

    pub fn withdraw_tokens_and_close(&mut self, bumps: &TakeBumps) -> Result<(), ProgramError> {
        let seeds = bumps.escrow_seeds();

        self.token_program
            .transfer(
                self.vault_token_account_for_mint_a,
                self.taker_token_account_for_mint_a,
                self.escrow,
                self.vault_token_account_for_mint_a.amount()
            )
            .invoke_signed(&seeds)?;

        self.token_program
            .close_account(self.vault_token_account_for_mint_a, self.taker, self.escrow)
            .invoke_signed(&seeds)
    }

    pub fn emit_event(&self) -> Result<(), ProgramError> {
        emit!(TakeEvent {
            escrow: *self.escrow.address(),
        });
        Ok(())
    }
}
