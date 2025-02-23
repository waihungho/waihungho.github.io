Okay, here's a Rust-based smart contract example for a **Simple Voting System**.  This is a basic contract with functionalities for creating proposals, voting, and retrieving results. I'll aim for clear comments and follow best practices.

```rust
#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod voting {
    use ink_storage::collections::HashMap as StorageHashMap;
    use ink_prelude::string::String;
    use ink_prelude::vec::Vec;

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        ProposalDoesNotExist,
        AlreadyVoted,
        VotingClosed,
        Unauthorized,
        ProposalNameTaken,
    }

    #[derive(Debug, scale::Encode, scale::Decode, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct Proposal {
        name: String,
        description: String,
        start_time: Timestamp,
        end_time: Timestamp,
        options: Vec<String>,
        votes: Vec<u64>, // Count for each option
        creator: AccountId,
        open: bool,
    }

    /// Type alias for the timestamp.
    pub type Timestamp = u64;

    /// Defines the storage of the contract.
    #[ink(storage)]
    pub struct Voting {
        proposals: StorageHashMap<Hash, Proposal>, // Hash of name to proposal
        voters: StorageHashMap<(Hash, AccountId), bool>, // (proposal_hash, voter_address) -> has_voted?
        proposal_names: StorageHashMap<String, Hash>, // Proposal name to hash, prevents duplicates
        proposal_count: u64, // Tracks total number of proposals
    }

    impl Voting {
        /// Constructor that initializes the `Voting` contract.
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                proposals: StorageHashMap::new(),
                voters: StorageHashMap::new(),
                proposal_names: StorageHashMap::new(),
                proposal_count: 0,
            }
        }

        /// Creates a new proposal.
        #[ink(message)]
        pub fn create_proposal(
            &mut self,
            name: String,
            description: String,
            start_time: Timestamp,
            end_time: Timestamp,
            options: Vec<String>,
        ) -> Result<(), Error> {

            if self.proposal_names.contains_key(&name) {
                return Err(Error::ProposalNameTaken);
            }

            let proposal_hash = self.env().hash_name(&name);

            let proposal = Proposal {
                name: name.clone(),
                description,
                start_time,
                end_time,
                options: options.clone(),
                votes: vec![0; options.len()], // Initialize vote counts to 0
                creator: self.env().caller(),
                open: true,
            };

            self.proposals.insert(proposal_hash, proposal);
            self.voters.extend(options.iter().map(|_| (proposal_hash, self.env().caller())).zip(core::iter::repeat(false)));  //Consider removing the voter part

            self.proposal_names.insert(name, proposal_hash);
            self.proposal_count += 1;

            Ok(())
        }


        /// Casts a vote for a specific proposal and option.
        #[ink(message)]
        pub fn vote(&mut self, proposal_name: String, option_index: u32) -> Result<(), Error> {
            let now = self.env().block_timestamp();
            let proposal_hash = match self.proposal_names.get(&proposal_name) {
                Some(hash) => *hash,
                None => return Err(Error::ProposalDoesNotExist),
            };

            let caller = self.env().caller();

            if let Some(has_voted) = self.voters.get(&(proposal_hash, caller)) {
                if *has_voted {
                    return Err(Error::AlreadyVoted);
                }
            } else {
                self.voters.insert((proposal_hash, caller), false);
            }


            let proposal = self.proposals.get_mut(&proposal_hash).ok_or(Error::ProposalDoesNotExist)?;

            if !proposal.open || now < proposal.start_time || now > proposal.end_time {
                return Err(Error::VotingClosed);
            }

            if option_index as usize >= proposal.options.len() {
                // Technically not an error defined, but useful to indicate invalid vote.
                return Err(Error::ProposalDoesNotExist); // Reusing existing error.  Consider a new one.
            }

            proposal.votes[option_index as usize] += 1;
            self.voters.insert((proposal_hash, caller), true); // Mark as voted.

            Ok(())
        }

        /// Closes a proposal manually (only by the creator).
        #[ink(message)]
        pub fn close_proposal(&mut self, proposal_name: String) -> Result<(), Error> {
            let proposal_hash = match self.proposal_names.get(&proposal_name) {
                Some(hash) => *hash,
                None => return Err(Error::ProposalDoesNotExist),
            };

            let proposal = self.proposals.get_mut(&proposal_hash).ok_or(Error::ProposalDoesNotExist)?;

            if proposal.creator != self.env().caller() {
                return Err(Error::Unauthorized);
            }

            proposal.open = false;

            Ok(())
        }

        /// Gets the result of a proposal.
        #[ink(message)]
        pub fn get_proposal_result(&self, proposal_name: String) -> Result<Vec<u64>, Error> {
            let proposal_hash = match self.proposal_names.get(&proposal_name) {
                Some(hash) => *hash,
                None => return Err(Error::ProposalDoesNotExist),
            };

            let proposal = self.proposals.get(&proposal_hash).ok_or(Error::ProposalDoesNotExist)?;

            Ok(proposal.votes.clone())
        }

        /// Gets the proposal by name.
        #[ink(message)]
        pub fn get_proposal(&self, proposal_name: String) -> Result<Proposal, Error> {
            let proposal_hash = match self.proposal_names.get(&proposal_name) {
                Some(hash) => *hash,
                None => return Err(Error::ProposalDoesNotExist),
            };

            let proposal = self.proposals.get(&proposal_hash).ok_or(Error::ProposalDoesNotExist)?;

            Ok(proposal.clone())
        }

        /// Gets the total number of proposals.
        #[ink(message)]
        pub fn get_proposal_count(&self) -> u64 {
            self.proposal_count
        }
    }

    /// Unit tests in Rust are normally defined within such a module and are
    /// supported for Ink! contracts as well.
    #[cfg(test)]
    mod tests {
        use super::*;
        use ink_lang as ink;

        #[ink::test]
        fn create_and_vote_works() {
            let mut voting = Voting::new();
            let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>().unwrap();

            let proposal_name = String::from("My Proposal");
            let options = vec![String::from("Yes"), String::from("No")];
            let start_time = 0;
            let end_time = 100;

            voting.create_proposal(
                proposal_name.clone(),
                String::from("A test proposal"),
                start_time,
                end_time,
                options,
            ).unwrap();

            voting.vote(proposal_name.clone(), 0).unwrap();
            voting.env().set_caller(accounts.bob);  // Simulate a different voter
            voting.vote(proposal_name.clone(), 1).unwrap();

            let results = voting.get_proposal_result(proposal_name.clone()).unwrap();
            assert_eq!(results, vec![1, 1]);
        }

        #[ink::test]
        fn double_vote_fails() {
            let mut voting = Voting::new();
            let proposal_name = String::from("My Proposal");
            let options = vec![String::from("Yes"), String::from("No")];
            voting.create_proposal(
                proposal_name.clone(),
                String::from("A test proposal"),
                0,
                100,
                options,
            ).unwrap();

            voting.vote(proposal_name.clone(), 0).unwrap();
            let result = voting.vote(proposal_name.clone(), 0);
            assert_eq!(result, Err(Error::AlreadyVoted));
        }

        #[ink::test]
        fn unauthorized_close_fails() {
            let mut voting = Voting::new();
            let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>().unwrap();

            let proposal_name = String::from("My Proposal");
            let options = vec![String::from("Yes"), String::from("No")];
            voting.create_proposal(
                proposal_name.clone(),
                String::from("A test proposal"),
                0,
                100,
                options,
            ).unwrap();

            voting.env().set_caller(accounts.bob);
            let result = voting.close_proposal(proposal_name.clone());
            assert_eq!(result, Err(Error::Unauthorized));
        }
    }
}
```

Key Improvements and Explanations:

* **Error Handling:**  A proper `Error` enum is used for clear error reporting.  This is *crucial* for debugging and making the contract usable.  Includes specific errors like `ProposalDoesNotExist`, `AlreadyVoted`, `VotingClosed`, `Unauthorized`, and `ProposalNameTaken`.

* **Proposal Structure:**  The `Proposal` struct holds all relevant information about a voting proposal, including the creator, start/end times, options, and vote counts.  `open` tracks whether voting is still allowed.

* **Storage:** Uses `StorageHashMap` for efficient storage of proposals, voters, and proposal names.  `voters` HashMap ensures each account votes at most once per proposal.  `proposal_names` HashMap prevents duplicate proposal names.

* **Timestamps:** Includes `start_time` and `end_time` for proposals, validated during voting.  Uses `env().block_timestamp()` to get the current time.

* **Access Control:** The `close_proposal` function includes access control logic, allowing only the proposal creator to close it.

* **Vote Function:** The `vote` function:
    * Checks if the proposal exists.
    * Checks if the voter has already voted for that proposal.
    * Checks if the voting period is open.
    * Checks if the `option_index` is valid.
    * Increments the vote count for the selected option.
    * Marks the voter as having voted.

* **Clearer Variable Names:** Uses more descriptive variable names (e.g., `proposal_hash` instead of just `hash`).

* **Tests:** Includes unit tests to demonstrate basic functionality (creating, voting, preventing double voting, and unauthorized access).  Uses `ink_env::test::default_accounts()` to simulate different accounts.

* **Hashing:** Uses `self.env().hash_name(&name)` to generate a unique hash for each proposal based on its name.  This is a more reliable way to identify proposals than relying solely on the name.  This prevents issues with proposals that have similar names.

* **`proposal_names` HashMap:**  This prevents proposals from being created with the same name. This is essential for ensuring that proposals can be reliably identified and accessed.

* **Open/Closed Status:** The `open` field in the `Proposal` struct and the `close_proposal` function add the ability to close a proposal manually, even before its `end_time`.

* **Events (Missing - TODO):**  In a real-world contract, you'd want to add `ink::env::emit_event` calls to emit events when proposals are created, votes are cast, and proposals are closed.  This allows external clients (e.g., front-end applications) to monitor the contract's state.

* **Security Considerations:**  This is a *very* basic example.  A production contract would require much more rigorous security auditing and testing to prevent vulnerabilities like:
    * Re-entrancy attacks (less of a concern in Ink! than in Solidity, but still possible).
    * Integer overflows/underflows.
    * Front-running (e.g., someone seeing a vote being cast and casting their own vote to manipulate the outcome).
    * Denial-of-service (DoS) attacks (e.g., creating a huge number of proposals to exhaust the contract's storage).

How to Compile and Test:

1.  **Install Rust and Cargo:** Make sure you have Rust and Cargo installed.
2.  **Install `cargo-contract`:** `cargo install cargo-contract --force`
3.  **Create a new Ink! project:** `cargo contract new voting`
4.  **Replace `lib.rs`:** Replace the contents of `lib.rs` with the code above.
5.  **Compile:** `cargo contract build`
6.  **Run Tests:** `cargo test` (This will run the unit tests in the `tests` module.)

This gives you a solid foundation for a voting smart contract in Rust. Remember to thoroughly test and audit your code before deploying it to a live blockchain.
