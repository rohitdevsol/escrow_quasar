#![cfg_attr(not(test), no_std)]

use quasar_lang::prelude::*;
pub mod instructions;
pub use instructions::*;
pub mod state;
pub mod event;
declare_id!("3AcEyc1RWdVrak9dvWigC3ppoSr2xhBeS8LkbPcgWZzg");

#[derive(Accounts)]
pub struct Initialize<'info> {
    pub payer: &'info mut Signer,
    pub system_program: &'info Program<System>,
}

impl<'info> Initialize<'info> {
    #[inline(always)]
    pub fn initialize(&self) -> Result<(), ProgramError> {
        Ok(())
    }
}

#[program]
mod escrow_quasar {
    use super::*;

    #[instruction(discriminator = 0)]
    pub fn initialize(ctx: Ctx<Initialize>) -> Result<(), ProgramError> {
        ctx.accounts.initialize()
    }
}

#[cfg(test)]
mod tests;
