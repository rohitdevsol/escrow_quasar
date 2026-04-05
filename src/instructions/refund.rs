use ::{ quasar_lang::prelude::*, quasar_spl::{ Mint, Token, TokenCpi } };
use crate::state::Escrow;
use crate::event::RefundEvent;
#[derive(Accounts)]
pub struct Refund<'info> {
    pub maker: &'info mut Signer,

    #[account(has_one = maker, close = maker, seeds = [b"escrow", maker], bump = escrow.bump)]
    pub escrow: &'info mut Account<Escrow>,
    pub mint_a: &'info Account<Mint>,

    #[account(init_if_needed, payer = maker, token::mint = mint_a, token::authority = maker)]
    pub maker_token_account_for_mint_a: &'info mut Account<Token>,
    pub vault_token_account_for_mint_a: &'info mut Account<Token>,

    // pub rent: &'info Sysvar<Rent>,
    pub token_program: &'info Program<Token>,
    pub system_program: &'info Program<System>,
}

impl<'info> Refund<'info> {
    #[inline(always)]
    pub fn withdraw_tokens_and_close(&mut self, bumps: &RefundBumps) -> Result<(), ProgramError> {
        let seeds = bumps.escrow_seeds();

        self.token_program
            .transfer(
                self.vault_token_account_for_mint_a,
                self.maker_token_account_for_mint_a,
                self.escrow,
                self.vault_token_account_for_mint_a.amount()
            )
            .invoke_signed(&seeds)?;

        self.token_program
            .close_account(self.vault_token_account_for_mint_a, self.maker, self.escrow)
            .invoke_signed(&seeds)
    }

    #[inline(always)]
    pub fn emit_event(&self) -> Result<(), ProgramError> {
        emit!(RefundEvent {
            escrow: *self.escrow.address(),
        });
        Ok(())
    }
}
