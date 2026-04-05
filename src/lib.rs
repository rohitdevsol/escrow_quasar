#![cfg_attr(not(test), no_std)]

use quasar_lang::prelude::*;
mod instructions;
use instructions::*;
mod state;
mod event;
declare_id!("3AcEyc1RWdVrak9dvWigC3ppoSr2xhBeS8LkbPcgWZzg");

#[program]
mod escrow_quasar {
    use super::*;

    #[instruction(discriminator = 0)]
    pub fn make(ctx: Ctx<Make>, deposit: u64, receive: u64) -> Result<(), ProgramError> {
        // store to escrow state buffer
        ctx.accounts.make_escrow(receive, &ctx.bumps)?;
        // emit the event
        ctx.accounts.emit_event(deposit, receive)?;
        // deposit to vault
        ctx.accounts.deposit_tokens(deposit)
    }

    #[instruction(discriminator = 1)]
    pub fn take(ctx: Ctx<Take>) -> Result<(), ProgramError> {
        // taker transfers token to maker ( amount mentioned in the escrow)
        ctx.accounts.transfer_tokens()?;
        // emit the event
        ctx.accounts.emit_event()?;
        // transfer from vault to taker .. everything .. and also send rent to taker after vault closing
        ctx.accounts.withdraw_tokens_and_close(&ctx.bumps)
    }

    #[instruction(discriminator = 2)]
    pub fn refund(ctx: Ctx<Refund>) -> Result<(), ProgramError> {
        // transfer from vault to maker .. everything .. and also send rent to maker after vault closing
        ctx.accounts.withdraw_tokens_and_close(&ctx.bumps)?;
        //emit the event
        ctx.accounts.emit_event()
    }
}

#[cfg(test)]
mod tests;
