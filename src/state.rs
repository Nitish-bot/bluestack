use quasar_lang::prelude::*;

#[account(discriminator = 1, set_inner)]
#[seeds(b"election", creator: Address)]
pub struct Election {
    pub creator: Address,
    pub winner: Option<Address>,
    pub candidates: Vec<Address, 3>,
    pub votes: Vec<u32, 3>,
}
