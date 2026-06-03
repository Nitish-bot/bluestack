use quasar_lang::prelude::*;

use crate::{errors::ElectionError, state::Election};

#[derive(Accounts)]
pub struct Vote {
    #[account(mut)]
    pub payer: Signer,
    #[account(mut)]
    pub election: Account<Election>,
}

impl Vote {
    #[inline(always)]
    pub fn handler(&mut self, candidate: Address) -> Result<(), ProgramError> {
        let mut election = self.election.as_mut(self.payer.to_account_view());

        let idx = election
            .candidates
            .iter()
            .position(|c| c == &candidate)
            .ok_or(ElectionError::InvalidCandidate)?;

        election.votes.as_mut()[idx] = election.votes[idx]
            .checked_add(PodU32::from(1))
            .ok_or(ProgramError::ArithmeticOverflow)?;

        Ok(())
    }
}
