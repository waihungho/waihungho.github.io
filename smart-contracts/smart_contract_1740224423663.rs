Okay, let's craft a simple Rust-based smart contract using the ink! framework. This one will implement a basic decentralized "Ballot Box" where users can submit a single vote (represented as a string) and query the current voting data. It prevents users from voting more than once.

```rust
#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod ballot_box {
    use ink_storage::{
        collections::HashMap as StorageHashMap,
        traits::{PackedLayout, SpreadLayout},
    };

    #[derive(scale::Encode, scale::Decode, Debug, Clone, PartialEq, Eq)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo)
    )]
    pub struct VoteData {
        vote: String,
        voter: AccountId,
    }

    /// Defines the storage of our contract.
    #[ink(storage)]
    pub struct BallotBox {
        /// Mapping from voter account ID to their vote.  Used to prevent multiple voting.
        votes: StorageHashMap<AccountId, VoteData>,
        /// Array of all unique votes cast.
        vote_options: StorageHashMap<String, u32>,
    }

    impl BallotBox {
        /// Constructor that initializes the `BallotBox` contract.
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                votes: StorageHashMap::new(),
                vote_options: StorageHashMap::new(),
            }
        }

        /// Allows a user to cast their vote.  Can only vote once.
        #[ink(message)]
        pub fn cast_vote(&mut self, vote: String) {
            let caller = self.env().caller();

            // Check if the user has already voted.
            if self.votes.contains_key(&caller) {
                panic!("You have already voted!");
            }

            // Record the vote
            let vote_data = VoteData{vote: vote.clone(), voter: caller};
            self.votes.insert(caller, vote_data);

            // Increment the vote option
            let current_count = self.vote_options.get(&vote).unwrap_or(&0).clone();
            self.vote_options.insert(vote, current_count + 1);
        }

        /// Returns the vote cast by a specific user (if they voted).
        #[ink(message)]
        pub fn get_vote(&self, account: AccountId) -> Option<String> {
            self.votes.get(&account).map(|vote_data| vote_data.vote.clone())
        }

        /// Returns the total votes for a given candidate
        #[ink(message)]
        pub fn get_vote_count(&self, candidate: String) -> u32 {
            *self.vote_options.get(&candidate).unwrap_or(&0)
        }

        /// Returns all vote options as a Vec
        #[ink(message)]
        pub fn get_all_vote_options(&self) -> Vec<(String, u32)> {
            self.vote_options.clone().into_iter().collect()
        }

        /// Returns the number of total votes cast.
        #[ink(message)]
        pub fn total_votes_cast(&self) -> u32 {
            self.votes.len() as u32
        }
    }

    /// Unit tests in Rust are normally defined under a `#[cfg(test)]` attribute.
    #[cfg(test)]
    mod tests {
        use super::*;
        use ink_lang as ink;

        #[ink::test]
        fn can_vote() {
            let mut ballot_box = BallotBox::new();
            let alice = AccountId::from([0x01; 32]);
            ballot_box.env().set_caller(alice);

            ballot_box.cast_vote("CandidateA".to_string());
            assert_eq!(ballot_box.get_vote(alice), Some("CandidateA".to_string()));
            assert_eq!(ballot_box.total_votes_cast(), 1);
        }

        #[ink::test]
        fn cannot_vote_twice() {
            let mut ballot_box = BallotBox::new();
            let alice = AccountId::from([0x01; 32]);
            ballot_box.env().set_caller(alice);

            ballot_box.cast_vote("CandidateA".to_string());

            let result = std::panic::catch_unwind(|| {
                ballot_box.cast_vote("CandidateB".to_string());
            });

            assert!(result.is_err());
        }

        #[ink::test]
        fn vote_counts_are_correct() {
            let mut ballot_box = BallotBox::new();
            let alice = AccountId::from([0x01; 32]);
            let bob = AccountId::from([0x02; 32]);

            ballot_box.env().set_caller(alice);
            ballot_box.cast_vote("CandidateA".to_string());

            ballot_box.env().set_caller(bob);
            ballot_box.cast_vote("CandidateB".to_string());

            assert_eq!(ballot_box.get_vote_count("CandidateA".to_string()), 1);
            assert_eq!(ballot_box.get_vote_count("CandidateB".to_string()), 1);
            assert_eq!(ballot_box.total_votes_cast(), 2);
        }
    }
}
```

Key improvements and explanations:

* **`VoteData` Struct:** This holds the actual vote string and the voter's `AccountId`.  Crucially, this links the vote to the voter, allowing us to prevent double voting.
* **`StorageHashMap<AccountId, VoteData>`:** This is the core data structure.  It maps each voter's `AccountId` to their `VoteData`. The `StorageHashMap` is crucial for persistent storage in the smart contract.
* **`StorageHashMap<String, u32>`:** This stores the count for each vote option.  We use a `String` as the key to represent the vote option (e.g., "CandidateA").
* **`cast_vote()`:**
    * Checks if the voter has already voted using `self.votes.contains_key(&caller)`.  If they have, it `panic!`s, which causes the transaction to revert and prevents the double vote.
    * Records the vote in the `votes` mapping by inserting the `caller`'s `AccountId` as the key and vote as the value.
    * Increments the appropriate count in the `vote_options` mapping.
* **`get_vote()`:** Retrieves a user's vote.  Returns an `Option<String>` to handle the case where the user hasn't voted.
* **`get_vote_count()`:** Retrieves the number of votes for a particular candidate.
* **`get_all_vote_options()`:**  Returns all the candidate options and their counts.  This is useful for displaying the results. Converts the hashmap to a vector of tuples for easier handling.
* **`total_votes_cast()`:** Returns the total number of votes.
* **Error Handling:** Uses `panic!` to revert the transaction if a user tries to vote twice.  This is the standard way to handle errors in ink!.
* **Tests:** Includes comprehensive unit tests to verify the contract's behavior, including the double-voting prevention.  Tests are crucial for smart contract development.  I added tests for `cannot_vote_twice` and `vote_counts_are_correct`.
* **Clearer Data Structures:** Using `VoteData` makes the contract more readable and easier to reason about.
* **Imports:** Explicitly imports the necessary types from `ink_storage`.
* **Conciseness and Readability:** The code is formatted for better readability.
* **Docstrings:**  Includes docstrings to explain the purpose of each function.

To use this contract:

1.  **Set up your environment:** Install Rust, `cargo-contract`, and other necessary tools as described in the ink! documentation: [https://use.ink/](https://use.ink/)
2.  **Compile:** `cargo contract build`
3.  **Deploy:** Deploy the `.contract` file to a Substrate-based chain (e.g., Polkadot, Kusama, a local development chain) using a tool like the Polkadot JS Apps UI.
4.  **Interact:** Use the Polkadot JS Apps UI or a custom application to call the `cast_vote`, `get_vote`, `get_vote_count`, `get_all_vote_options`, and `total_votes_cast` functions.

This example provides a basic, functional smart contract that addresses the specified requirements. Remember to thoroughly test and audit your smart contracts before deploying them to a live blockchain environment.  Also consider access control mechanisms if only certain accounts should be able to call certain functions (e.g., a function to close the ballot box).
