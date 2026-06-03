// Minimal Quasar counter — PDA init/validate uses address = Type::seeds(args)

#![no_std]

use quasar_lang::prelude::*;

declare_id!("22222222222222222222222222222222222222222222");

#[account(discriminator = 1, set_inner)]
#[seeds(b"counter", authority: Address)]
pub struct Counter {
    pub authority: Address,
    pub count: u64,
    pub bump: u8,
}

#[derive(Accounts)]
pub struct Initialize {
    #[account(mut)]
    pub authority: Signer,
    #[account(mut, init, payer = authority, address = Counter::seeds(authority.address()))]
    pub counter: Account<Counter>,
    pub system_program: Program<SystemProgram>,
}

#[derive(Accounts)]
pub struct Increment {
    #[account(mut, has_one(authority), address = Counter::seeds(authority.address()))]
    pub counter: Account<Counter>,
    pub authority: Signer,
}

#[program]
mod counter_program {
    use super::*;

    #[instruction(discriminator = 0)]
    pub fn initialize(ctx: Ctx<Initialize>) -> Result<(), ProgramError> {
        ctx.accounts.counter.set_inner(CounterInner {
            authority: *ctx.accounts.authority.address(),
            count: 0,
            bump: ctx.bumps.counter,
        });
        Ok(())
    }

    #[instruction(discriminator = 1)]
    pub fn increment(ctx: Ctx<Increment>) -> Result<(), ProgramError> {
        ctx.accounts.counter.count += 1;
        Ok(())
    }
}
