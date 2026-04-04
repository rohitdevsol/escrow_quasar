use quasar_lang::prelude::*;
use crate::state::Escrow;

pub struct Make<'info> {
    pub maker: &'info mut Signer,

    pub escrow: &'info mut Account<Escrow>,
}
