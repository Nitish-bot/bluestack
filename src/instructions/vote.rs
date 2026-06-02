use quasar_lang::prelude::*;

use crate::state::Election;

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

        let mut idx = 0usize;
        for candi in election.candidates.iter() {
            if candi == &candidate {
                break;
            }
            idx += 1;
        }

        let votes = election.votes.as_mut();
        votes[idx] += 1;

        Ok(())
    }
}
