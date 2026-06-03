use quasar_lang::prelude::*;

use crate::{errors::ElectionError, state::Election};

#[derive(Accounts)]
pub struct DeclareWinner {
    pub payer: Signer,
    #[account(
        mut,
        constraints(election.creator == *payer.address()) @ ElectionError::Unauthorized,
    )]
    pub election: Account<Election>,
}

impl DeclareWinner {
    pub fn handler(&mut self) -> Result<(), ProgramError> {
        let votes = self.election.votes();
        let idx = votes
            .iter()
            .enumerate()
            .fold(0usize, |best, (i, v)| if *v > votes[best] { i } else { best });

        let winner = self.election.candidates()[idx];
        self.election.winner.set(Some(winner));
        Ok(())
    }
}
