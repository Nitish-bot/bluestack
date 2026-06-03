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

fn election_from_result(result: &quasar_svm::ExecutionResult, election: Pubkey) -> DecodedElection {
    let acct = result
        .account(&election)
        .expect("election account missing from result");
    decode_election(&acct.data)
}

fn election_from_accounts(accounts: &[Account], election: Pubkey) -> DecodedElection {
    let acct = accounts
        .iter()
        .find(|a| a.address == election)
        .expect("election account missing from accounts");
    decode_election(&acct.data)
}

/// Generated client uses u32 length prefix; on-chain `Vec<Address, 3>` uses u16.
fn create_election_ix(payer: Pubkey, election: Pubkey, candidates: [Pubkey; 3]) -> Instruction {
    let candidate_addrs = vec![
        to_address(candidates[0]),
        to_address(candidates[1]),
        to_address(candidates[2]),
    ];

    let mut ix: Instruction = CreateElectionInstruction {
        payer: to_address(payer),
        election: to_address(election),
        rent: to_address(quasar_svm::solana_sdk_ids::sysvar::rent::ID),
        system_program: to_address(quasar_svm::system_program::ID),
        candidates: candidate_addrs.clone(),
    }
    .into();

    let mut data = vec![0u8];
    data.extend_from_slice(&(candidate_addrs.len() as u16).to_le_bytes());
    for addr in candidate_addrs {
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

    let payer = Pubkey::new_unique();
    let election = election_pda(payer);
    let candidates: [Pubkey; 3] = std::array::from_fn(|_| Pubkey::new_unique());

    let ix = create_election_ix(payer, election, candidates);
    let result = svm.process_instruction(&ix, &[signer(payer), empty(election)]);
    assert!(
        result.is_ok(),
        "create_election failed: {:?}",
        result.raw_result
    );

    let state = election_from_result(&result, election);
    assert_eq!(state.creator, payer);
    assert_eq!(state.winner, None);
    assert_eq!(state.candidates, candidates.as_slice());
    assert_eq!(state.votes, [0, 0, 0]);
}

#[test]
fn test_happy_path_votes_and_declare_winner() {
    let mut svm = setup();

    let payer = Pubkey::new_unique();
    let election = election_pda(payer);
    let candidates: [Pubkey; 3] = std::array::from_fn(|_| Pubkey::new_unique());

    let create_ix = create_election_ix(payer, election, candidates);
    let mut accounts = vec![signer(payer), empty(election)];

    let create_result = svm.process_instruction(&create_ix, &accounts);
    assert!(
        create_result.is_ok(),
        "create_election failed: {:?}",
        create_result.raw_result
    );
    accounts = create_result.accounts;

    let vote_counts = [3u32, 1, 2];
    for (i, &count) in vote_counts.iter().enumerate() {
        for _ in 0..count {
            let ix = vote_ix(payer, election, candidates[i]);
            let result = svm.process_instruction(&ix, &accounts);
            assert!(
                result.is_ok(),
                "vote for candidate {i} failed: {:?}",
                result.raw_result
            );
            accounts = result.accounts;
        }
    }

    let state = election_from_accounts(&accounts, election);
    assert_eq!(state.votes, vote_counts.as_slice());

    let declare_ix = declare_winner_ix(payer, election);
    let declare_result = svm.process_instruction(&declare_ix, &accounts);
    assert!(
        declare_result.is_ok(),
        "declare_winner failed: {:?}",
        declare_result.raw_result
    );

    let state = election_from_result(&declare_result, election);
    assert_eq!(state.winner, Some(candidates[0]));
    assert_eq!(state.votes, vote_counts.as_slice());
}

#[test]
fn test_tie_lowest_index_wins() {
    let mut svm = setup();

    let payer = Pubkey::new_unique();
    let election = election_pda(payer);
    let candidates: [Pubkey; 3] = std::array::from_fn(|_| Pubkey::new_unique());

    let create_ix = create_election_ix(payer, election, candidates);
    let mut accounts = vec![signer(payer), empty(election)];

    let create_result = svm.process_instruction(&create_ix, &accounts);
    assert!(
        create_result.is_ok(),
        "create_election failed: {:?}",
        create_result.raw_result
    );
    accounts = create_result.accounts;

    let tie_votes = [2u32, 2, 1];
    for (i, &count) in tie_votes.iter().enumerate() {
        for _ in 0..count {
            let ix = vote_ix(payer, election, candidates[i]);
            let result = svm.process_instruction(&ix, &accounts);
            assert!(
                result.is_ok(),
                "vote (tie) candidate {i} failed: {:?}",
                result.raw_result
            );
            accounts = result.accounts;
        }
    }

    let declare_ix = declare_winner_ix(payer, election);
    let declare_result = svm.process_instruction(&declare_ix, &accounts);
    assert!(
        declare_result.is_ok(),
        "declare_winner (tie) failed: {:?}",
        declare_result.raw_result
    );

    let state = election_from_result(&declare_result, election);
    assert_eq!(state.winner, Some(candidates[0]));
    assert_eq!(state.votes, tie_votes.as_slice());
}

#[test]
fn test_no_votes_first_candidate_wins() {
    let mut svm = setup();

    let payer = Pubkey::new_unique();
    let election = election_pda(payer);
    let candidates: [Pubkey; 3] = std::array::from_fn(|_| Pubkey::new_unique());

    let create_ix = create_election_ix(payer, election, candidates);
    let mut accounts = vec![signer(payer), empty(election)];

    let create_result = svm.process_instruction(&create_ix, &accounts);
    assert!(
        create_result.is_ok(),
        "create_election failed: {:?}",
        create_result.raw_result
    );
    accounts = create_result.accounts;

    let declare_ix = declare_winner_ix(payer, election);
    let declare_result = svm.process_instruction(&declare_ix, &accounts);
    assert!(
        declare_result.is_ok(),
        "declare_winner (no votes) failed: {:?}",
        declare_result.raw_result
    );

    let state = election_from_result(&declare_result, election);
    assert_eq!(state.winner, Some(candidates[0]));
    assert_eq!(state.votes, [0, 0, 0]);
}

#[test]
fn test_vote_non_candidate_fails() {
    let mut svm = setup();

    let payer = Pubkey::new_unique();
    let election = election_pda(payer);
    let candidates: [Pubkey; 3] = std::array::from_fn(|_| Pubkey::new_unique());
    let stranger = Pubkey::new_unique();

    let create_ix = create_election_ix(payer, election, candidates);
    let mut accounts = vec![signer(payer), empty(election)];

    let create_result = svm.process_instruction(&create_ix, &accounts);
    assert!(
        create_result.is_ok(),
        "create_election failed: {:?}",
        create_result.raw_result
    );
    accounts = create_result.accounts;

    let bad_vote_ix = vote_ix(payer, election, stranger);
    let result = svm.process_instruction(&bad_vote_ix, &accounts);
    assert!(result.is_err(), "expected vote for non-candidate to fail");
}
