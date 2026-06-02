use quasar_lang::prelude::*;

use crate::state::Election;

#[derive(Accounts)]
pub struct DeclareWinner {
    #[account(mut)]
    pub payer: Signer,
    #[account(mut)]
    pub election: Account<Election>,
    pub system_program: Program<SystemProgram>,
}

impl DeclareWinner {
    #[inline(always)]
    pub fn handler(&mut self) -> Result<(), ProgramError> {
        let votes = self.election.votes();

        let mut idx = 0usize;
        for (i, vote) in votes.iter().enumerate() {
            if *vote > votes[idx] {
                idx = i;
            }
        }

        let winner = self.election.candidates()[idx];
        self.election.winner.set(Some(winner));

        Ok(())
    }
}
