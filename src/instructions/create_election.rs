use quasar_lang::prelude::*;

use crate::state::{Election, ElectionInner};

#[derive(Accounts)]
pub struct CreateElection {
    #[account(mut)]
    pub payer: Signer,
    #[account(
        init,
        payer = payer,
        address = Election::seeds(payer.address()),
    )]
    pub election: Account<Election>,
    pub rent: Sysvar<Rent>,
    pub system_program: Program<SystemProgram>,
}

impl CreateElection {
    #[inline(always)]
    pub fn handler(&mut self, candidates: &[Address]) -> Result<(), ProgramError> {
        let votes = [PodU32::from(0u32); 3];

        self.election.set_inner(
            ElectionInner {
                creator: *self.payer.address(),
                winner: None,
                candidates,
                votes: votes.as_ref(),
            },
            self.payer.to_account_view(),
            self.rent.lamports_per_byte(),
            self.rent.exemption_threshold_raw(),
        )
    }
}
