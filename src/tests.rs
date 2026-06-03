extern crate std;

use {
    alloc::vec,
    bluestack_client::*,
    quasar_svm::{Account, Instruction, Pubkey, QuasarSvm},
    solana_address::Address,
};

const ELECTION_DISCRIMINATOR: [u8; 1] = [1];
const LAMPORTS: u64 = 10_000_000_000;

struct DecodedElection {
    creator: Pubkey,
    winner: Option<Pubkey>,
    candidates: Vec<Pubkey>,
    votes: Vec<u32>,
}

struct ElectionFixture {
    payer: Pubkey,
    election: Pubkey,
    candidates: [Pubkey; 3],
    accounts: Vec<Account>,
}

impl ElectionFixture {
    fn create(svm: &mut QuasarSvm) -> Self {
        let payer = Pubkey::new_unique();
        let election = election_pda(payer);
        let candidates = std::array::from_fn(|_| Pubkey::new_unique());
        let result = svm.process_instruction(
            &create_election_ix(payer, election, candidates),
            &[signer(payer), empty(election)],
        );
        assert!(
            result.is_ok(),
            "create_election failed: {:?}",
            result.raw_result
        );
        Self {
            payer,
            election,
            candidates,
            accounts: result.accounts,
        }
    }

    fn cast_votes(&mut self, svm: &mut QuasarSvm, counts: [u32; 3]) {
        for (i, &count) in counts.iter().enumerate() {
            for _ in 0..count {
                let result = svm.process_instruction(
                    &vote_ix(self.payer, self.election, self.candidates[i]),
                    &self.accounts,
                );
                assert!(
                    result.is_ok(),
                    "vote for candidate {i} failed: {:?}",
                    result.raw_result
                );
                self.accounts = result.accounts;
            }
        }
    }

    fn declare_winner(&mut self, svm: &mut QuasarSvm) -> DecodedElection {
        let result = svm.process_instruction(
            &declare_winner_ix(self.payer, self.election),
            &self.accounts,
        );
        assert!(
            result.is_ok(),
            "declare_winner failed: {:?}",
            result.raw_result
        );
        decode_election_at(&result.accounts, self.election)
    }

    fn state(&self) -> DecodedElection {
        decode_election_at(&self.accounts, self.election)
    }
}

fn setup() -> QuasarSvm {
    let elf = std::fs::read("target/deploy/bluestack.so").unwrap();
    QuasarSvm::new().with_program(&crate::ID, &elf)
}

fn to_address(pk: Pubkey) -> Address {
    Address::new_from_array(pk.to_bytes())
}

fn signer(pubkey: Pubkey) -> Account {
    quasar_svm::token::create_keyed_system_account(&pubkey, LAMPORTS)
}

fn empty(pubkey: Pubkey) -> Account {
    Account {
        address: pubkey,
        lamports: 0,
        data: vec![],
        owner: quasar_svm::system_program::ID,
        executable: false,
    }
}

fn election_pda(creator: Pubkey) -> Pubkey {
    let (pda, _) = Pubkey::find_program_address(&[b"election", creator.as_ref()], &crate::ID);
    pda
}

fn decode_election(data: &[u8]) -> DecodedElection {
    use crate::state::__election_zc::__SchemaRef;

    assert!(
        data.len() >= ELECTION_DISCRIMINATOR.len(),
        "election account data too short"
    );
    for (i, &b) in ELECTION_DISCRIMINATOR.iter().enumerate() {
        assert_eq!(data[i], b, "invalid election discriminator");
    }

    let election = __SchemaRef::new(&data[ELECTION_DISCRIMINATOR.len()..])
        .expect("invalid election compact layout");

    DecodedElection {
        creator: Pubkey::new_from_array(election.creator.to_bytes()),
        winner: election
            .winner
            .get()
            .map(|addr| Pubkey::new_from_array(addr.to_bytes())),
        candidates: election
            .candidates()
            .iter()
            .map(|addr| Pubkey::new_from_array(addr.to_bytes()))
            .collect(),
        votes: election.votes().iter().map(|v| u32::from(*v)).collect(),
    }
}

fn decode_election_at(accounts: &[Account], election: Pubkey) -> DecodedElection {
    let data = &accounts
        .iter()
        .find(|a| a.address == election)
        .expect("election account missing")
        .data;
    decode_election(data)
}

/// Generated client uses u32 length prefix; on-chain `Vec<Address, 3>` uses u16.
fn create_election_ix(payer: Pubkey, election: Pubkey, candidates: [Pubkey; 3]) -> Instruction {
    let addrs: Vec<Address> = candidates.iter().map(|&pk| to_address(pk)).collect();

    let mut ix: Instruction = CreateElectionInstruction {
        payer: to_address(payer),
        election: to_address(election),
        rent: to_address(quasar_svm::solana_sdk_ids::sysvar::rent::ID),
        system_program: to_address(quasar_svm::system_program::ID),
        candidates: addrs.clone(),
    }
    .into();

    let mut data = vec![0u8];
    data.extend_from_slice(&(addrs.len() as u16).to_le_bytes());
    for addr in &addrs {
        data.extend_from_slice(addr.as_ref());
    }
    ix.data = data;
    ix
}

fn vote_ix(payer: Pubkey, election: Pubkey, candidate: Pubkey) -> Instruction {
    VoteInstruction {
        payer: to_address(payer),
        election: to_address(election),
        candidate: to_address(candidate),
    }
    .into()
}

fn declare_winner_ix(payer: Pubkey, election: Pubkey) -> Instruction {
    DeclareWinnerInstruction {
        payer: to_address(payer),
        election: to_address(election),
    }
    .into()
}

#[test]
fn test_create_election() {
    let mut svm = setup();
    let fx = ElectionFixture::create(&mut svm);
    let state = fx.state();

    assert_eq!(state.creator, fx.payer);
    assert_eq!(state.winner, None);
    assert_eq!(state.candidates, fx.candidates.as_slice());
    assert_eq!(state.votes, [0, 0, 0]);
}

#[test]
fn test_happy_path_votes_and_declare_winner() {
    let mut svm = setup();
    let mut fx = ElectionFixture::create(&mut svm);
    let vote_counts = [3u32, 1, 2];

    fx.cast_votes(&mut svm, vote_counts);
    assert_eq!(fx.state().votes, vote_counts.as_slice());

    let state = fx.declare_winner(&mut svm);
    assert_eq!(state.winner, Some(fx.candidates[0]));
    assert_eq!(state.votes, vote_counts.as_slice());
}

#[test]
fn test_tie_lowest_index_wins() {
    let mut svm = setup();
    let mut fx = ElectionFixture::create(&mut svm);
    let tie_votes = [2u32, 2, 1];

    fx.cast_votes(&mut svm, tie_votes);
    let state = fx.declare_winner(&mut svm);

    assert_eq!(state.winner, Some(fx.candidates[0]));
    assert_eq!(state.votes, tie_votes.as_slice());
}

#[test]
fn test_no_votes_first_candidate_wins() {
    let mut svm = setup();
    let mut fx = ElectionFixture::create(&mut svm);
    let state = fx.declare_winner(&mut svm);

    assert_eq!(state.winner, Some(fx.candidates[0]));
    assert_eq!(state.votes, [0, 0, 0]);
}

#[test]
fn test_vote_non_candidate_fails() {
    let mut svm = setup();
    let fx = ElectionFixture::create(&mut svm);
    let stranger = Pubkey::new_unique();

    let result = svm.process_instruction(
        &vote_ix(fx.payer, fx.election, stranger),
        &fx.accounts,
    );
    assert!(result.is_err(), "expected vote for non-candidate to fail");
}
