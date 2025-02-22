Okay, here's a Rust smart contract that implements a simple decentralized voting system using ink!. This example focuses on simplicity and demonstrates key concepts like storage, state transitions, and events.  It allows anyone to create a proposal, and registered participants can vote on them.

```rust
#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod voting {
    use ink_storage::{
        collections::HashMap as StorageHashMap,
        traits::{PackedLayout, SpreadLayout},
    };

    /// Defines the storage struct of this contract.
    #[ink(storage)]
    pub struct Voting {
        /// A map from proposal ID to its description.
        proposals: StorageHashMap<u64, Proposal>,
        /// A map from proposal ID to a map of voters and their votes (true for yes, false for no).
        votes: StorageHashMap<u64, StorageHashMap<AccountId, bool>>,
        /// A set of registered voters allowed to vote.
        registered_voters: StorageHashMap<AccountId, bool>,
        /// Unique proposal ID counter.
        proposal_id_counter: u64,
        /// Owner of contract, can register voters.
        owner: AccountId,
    }

    /// Represents a proposal in the voting system.
    #[derive(Clone, Debug, PartialEq, scale::Encode, scale::Decode, SpreadLayout, PackedLayout)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo)
    )]
    pub struct Proposal {
        description: String,
        creator: AccountId,
        // Add other relevant proposal details here if needed (e.g., deadline, etc.)
    }

    /// Event emitted when a new proposal is created.
    #[ink(event)]
    pub struct ProposalCreated {
        #[ink(topic)]
        proposal_id: u64,
        creator: AccountId,
        description: String,
    }

    /// Event emitted when a voter casts their vote.
    #[ink(event)]
    pub struct VoteCast {
        #[ink(topic)]
        proposal_id: u64,
        voter: AccountId,
        vote: bool, // True for yes, False for no
    }

    /// Event emitted when a voter is registered
    #[ink(event)]
    pub struct VoterRegistered {
        voter: AccountId
    }

    /// Errors that can occur during contract execution.
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        ProposalDoesNotExist,
        AlreadyVoted,
        VoterNotRegistered,
        NotOwner,
    }

    impl Voting {
        /// Constructor that initializes the voting system.
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                proposals: StorageHashMap::new(),
                votes: StorageHashMap::new(),
                registered_voters: StorageHashMap::new(),
                proposal_id_counter: 0,
                owner: Self::env().caller(),
            }
        }

        /// Creates a new proposal.
        #[ink(message)]
        pub fn create_proposal(&mut self, description: String) -> u64 {
            let proposal_id = self.proposal_id_counter;
            let caller = self.env().caller();

            let proposal = Proposal {
                description: description.clone(),
                creator: caller,
            };

            self.proposals.insert(proposal_id, proposal);
            self.votes.insert(proposal_id, StorageHashMap::new()); // Initialize votes for the proposal.
            self.proposal_id_counter += 1;

            self.env().emit_event(ProposalCreated {
                proposal_id,
                creator: caller,
                description,
            });

            proposal_id
        }

        /// Registers a voter. Only the owner can do this.
        #[ink(message)]
        pub fn register_voter(&mut self, voter: AccountId) -> Result<(), Error> {
            if self.env().caller() != self.owner {
                return Err(Error::NotOwner);
            }
            self.registered_voters.insert(voter, true);
            self.env().emit_event(VoterRegistered { voter });
            Ok(())
        }

        /// Allows a registered voter to cast a vote on a proposal.
        #[ink(message)]
        pub fn vote(&mut self, proposal_id: u64, vote: bool) -> Result<(), Error> {
            if !self.proposals.contains_key(&proposal_id) {
                return Err(Error::ProposalDoesNotExist);
            }

            let caller = self.env().caller();

            if !self.registered_voters.contains_key(&caller) {
                return Err(Error::VoterNotRegistered);
            }

            let proposal_votes = self.votes.get_mut(&proposal_id).expect("Proposal votes must exist");

            if proposal_votes.contains_key(&caller) {
                return Err(Error::AlreadyVoted);
            }

            proposal_votes.insert(caller, vote);

            self.env().emit_event(VoteCast {
                proposal_id,
                voter: caller,
                vote,
            });

            Ok(())
        }

        /// Gets the vote count for a specific proposal.
        #[ink(message)]
        pub fn get_vote_count(&self, proposal_id: u64) -> Result<(u64, u64), Error> {
            if !self.proposals.contains_key(&proposal_id) {
                return Err(Error::ProposalDoesNotExist);
            }

            let proposal_votes = self.votes.get(&proposal_id).expect("Proposal votes must exist");

            let mut yes_count: u64 = 0;
            let mut no_count: u64 = 0;

            for (_voter, vote) in proposal_votes.iter() {
                if *vote {
                    yes_count += 1;
                } else {
                    no_count += 1;
                }
            }

            Ok((yes_count, no_count))
        }

        /// Gets the proposal by id.
        #[ink(message)]
        pub fn get_proposal(&self, proposal_id: u64) -> Option<Proposal> {
            self.proposals.get(&proposal_id).cloned()
        }

        /// Get the owner of the contract.
        #[ink(message)]
        pub fn get_owner(&self) -> AccountId {
            self.owner
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink_lang as ink;

        #[ink::test]
        fn create_and_vote_works() {
            let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>().expect("Cannot get accounts");
            let mut voting = Voting::new();

            // Register voter
            voting.register_voter(accounts.alice).unwrap();

            // Create a proposal
            let proposal_id = voting.create_proposal("Test Proposal".to_string());

            // Alice votes "yes"
            voting.vote(proposal_id, true).unwrap();

            // Check the vote count
            let (yes_count, no_count) = voting.get_vote_count(proposal_id).unwrap();
            assert_eq!(yes_count, 1);
            assert_eq!(no_count, 0);
        }

        #[ink::test]
        fn vote_twice_fails() {
            let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>().expect("Cannot get accounts");
            let mut voting = Voting::new();
            voting.register_voter(accounts.alice).unwrap();


            let proposal_id = voting.create_proposal("Test Proposal".to_string());

            voting.vote(proposal_id, true).unwrap();

            let result = voting.vote(proposal_id, false); // Try to vote again
            assert_eq!(result, Err(Error::AlreadyVoted));
        }

        #[ink::test]
        fn vote_unregistered_fails() {
            let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>().expect("Cannot get accounts");
            let mut voting = Voting::new();

            let proposal_id = voting.create_proposal("Test Proposal".to_string());

            let result = voting.vote(proposal_id, true); // Try to vote again
            assert_eq!(result, Err(Error::VoterNotRegistered));
        }

        #[ink::test]
        fn only_owner_can_register_voter() {
            let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>().expect("Cannot get accounts");
            let mut voting = Voting::new();
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(accounts.bob);
            let result = voting.register_voter(accounts.alice);

            assert_eq!(result, Err(Error::NotOwner));
        }

        #[ink::test]
        fn get_proposal_works() {
           let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>().expect("Cannot get accounts");
            let mut voting = Voting::new();
            let proposal_id = voting.create_proposal("Test Proposal".to_string());
            let proposal = voting.get_proposal(proposal_id).unwrap();

            assert_eq!(proposal.description, "Test Proposal".to_string());
            assert_eq!(proposal.creator, accounts.alice);
        }

    }
}
```

Key improvements and explanations:

* **Clear Structure:** The code is well-organized with comments explaining each part.  The `Voting` struct holds the core state.
* **Error Handling:** The `Error` enum defines possible error conditions, making debugging easier.  Functions return `Result` to indicate success or failure.
* **Events:**  Events are emitted when important actions occur (proposal creation, voting).  This allows external observers to track activity on the contract.
* **Registered Voters:**  Now, only registered voters can participate.  This prevents spam voting and can be a requirement for many real-world voting scenarios.  The `register_voter` function adds an address to the `registered_voters` map.
* **Owner Role:** The `register_voter` function is protected so that only the contract owner can register new voters.  This is a common access control pattern.
* **Voting Logic:**  The `vote` function:
    * Checks if the proposal exists.
    * Checks if the voter is registered.
    * Prevents double-voting.
    * Updates the `votes` map.
    * Emits a `VoteCast` event.
* **Vote Counting:** The `get_vote_count` function efficiently counts the votes for a proposal, returning the yes and no counts.
* **Get Proposal by ID:**  Added a `get_proposal` function to retrieve proposal details.
* **Tests:** Comprehensive tests cover:
    * Creating proposals and voting.
    * Preventing double-voting.
    * Preventing unregistered users from voting.
    * Testing the owner-only registration.
    * Testing the get_proposal function.
* **Dependencies:** Includes the necessary `ink_lang` and `ink_storage` dependencies.
* **`no_std` Support:**  Includes `#![cfg_attr(not(feature = "std"), no_std)]` to make the contract compatible with environments without the standard library.
* **`scale` and `scale_info`:** Includes the necessary derives for SCALE encoding/decoding and type information. This is essential for ink! contracts.
* **StorageHashMap usage:**  Uses `StorageHashMap` from `ink_storage` for persistent storage in the contract.

How to compile and deploy (basic steps):

1. **Install `cargo contract`:**
   ```bash
   cargo install cargo-contract --force --locked
   ```

2. **Build the contract:**
   ```bash
   cargo contract build
   ```
   This will create a `.contract` file in the `target/ink` directory.  This file contains the Wasm bytecode and metadata needed to deploy the contract.

3. **Deploy using a tool like `Polkadot JS Apps`:**  Connect to a development chain (e.g., `substrate-node-template`) and use the Polkadot JS Apps UI to deploy the `.contract` file.  You'll need to instantiate the contract using the `new` constructor.

4. **Interact with the contract:**  Use the Polkadot JS Apps UI or write a client-side application (e.g., using `polkadot-js/api`) to call the contract's functions (e.g., `create_proposal`, `vote`, `get_vote_count`).

This comprehensive example provides a solid foundation for building more complex decentralized voting systems on ink!.  You can extend it with features like:

* **Time-limited voting:** Add a deadline to proposals.
* **Different voting options:**  Support ranked-choice voting or other voting systems.
* **Delegated voting:**  Allow voters to delegate their vote to another address.
* **Privacy:**  Implement zero-knowledge proofs to make votes private.
* **More robust access control:** Implement roles beyond just the owner (e.g., moderators).
* **Token-weighted voting:**  Allow users to vote with different weights based on the amount of a specific token they hold.
