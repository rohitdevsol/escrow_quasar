use quasar_lang::prelude::*;

#[account(discriminator = 1)]
pub struct Escrow {
    pub maker: Address,
    pub mint_a: Address,
    pub mint_b: Address,
    pub maker_token_account_for_mint_b: Address,
    pub receive: u64,
    pub bump: u8,
}
