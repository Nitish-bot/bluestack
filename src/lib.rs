#![cfg_attr(not(test), no_std)]
#![allow(dead_code)]
use quasar_lang::prelude::*;

mod errors;
mod instructions;
mod state;
#[cfg(test)]
mod tests;
use instructions::*;

declare_id!("4ZmkkesWXMKvVKrrwxAz88sPYivrKevveg6pEPWmuDfW");

#[program]
mod bluestack {
    use super::*;

    #[instruction(discriminator = 0)]
    pub fn create_election(
        ctx: Ctx<CreateElection>,
        candidates: Vec<Address, 3>,
    ) -> Result<(), ProgramError> {
        ctx.accounts.handler(candidates)
    }

    #[instruction(discriminator = 1, heap)]
    pub fn vote(ctx: Ctx<Vote>, candidate: Address) -> Result<(), ProgramError> {
        ctx.accounts.handler(candidate)
    }

    #[instruction(discriminator = 2)]
    pub fn declare_winner(ctx: Ctx<DeclareWinner>) -> Result<(), ProgramError> {
        ctx.accounts.handler()
    }
}
