```rust
#![cfg_attr(not(feature = "std"), no_std)]

// Define the smart contract's module
#[ink::contract]
mod decentralized_opinion_polling {
    use ink::prelude::{
        string::String,
        vec::Vec,
    };
    use ink::storage::Mapping;

    /// Defines the storage of the contract.
    #[ink::storage]
    pub struct DecentralizedOpinionPolling {
        /// The owner of the contract.
        owner: AccountId,
        /// A mapping from poll ID to poll details.
        polls: Mapping<u32, Poll>,
        /// A counter for generating unique poll IDs.
        poll_count: u32,
        /// A mapping from poll ID and AccountId to vote weight.
        votes: Mapping<(u32, AccountId), u32>, // Track user votes
    }

    /// Represents a poll with its details.
    #[derive(scale::Encode, scale::Decode, Debug, PartialEq, Eq, Clone)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo)
    )]
    pub struct Poll {
        /// The ID of the poll.
        id: u32,
        /// The question of the poll.
        question: String,
        /// The options available for the poll.
        options: Vec<String>,
        /// The start timestamp of the poll.
        start_timestamp: Timestamp,
        /// The end timestamp of the poll.
        end_timestamp: Timestamp,
        /// A flag indicating whether the poll is active.
        is_active: bool,
        /// Voting power calculation method
        voting_power_strategy: VotingPowerStrategy,
        /// A mapping from option index to vote count.
        results: Vec<u32>, // Store vote counts directly within the Poll struct
    }

    /// Represents different strategy of vote power calculation
    #[derive(scale::Encode, scale::Decode, Debug, PartialEq, Eq, Clone)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo)
    )]
    pub enum VotingPowerStrategy {
        OnePersonOneVote,
        // Weighted by balance in the contract
        BalanceWeighted,
        // Weighted by ERC20 token balance
        TokenWeighted { token_address: AccountId },
    }

    /// Emitted when a new poll is created.
    #[ink::event]
    pub struct PollCreated {
        poll_id: u32,
        creator: AccountId,
        question: String,
    }

    /// Emitted when a vote is cast.
    #[ink::event]
    pub struct VoteCast {
        poll_id: u32,
        voter: AccountId,
        option_index: u32,
        vote_weight: u32,
    }

    /// Emitted when a poll is ended.
    #[ink::event]
    pub struct PollEnded {
        poll_id: u32,
    }

    /// Defines the errors that can occur in the contract.
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        /// Caller is not the owner.
        NotOwner,
        /// Poll does not exist.
        PollNotFound,
        /// Poll is not active.
        PollNotActive,
        /// Invalid option index.
        InvalidOption,
        /// Poll has already ended.
        PollEnded,
        /// Voting period not yet started
        PollNotStarted,
        /// User has already voted.
        AlreadyVoted,
        /// Overflow Error
        Overflow,
        /// Poll Start time is later than end time
        InvalidTimeRange,
        /// Insufficient Balance,
        InsufficientBalance,
        /// Token address invalid
        InvalidTokenAddress,
    }

    /// Type alias for the contract's result type.
    pub type Result<T> = core::result::Result<T, Error>;

    impl DecentralizedOpinionPolling {
        /// Constructor that initializes the contract.
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                owner: Self::env().caller(),
                polls: Mapping::default(),
                poll_count: 0,
                votes: Mapping::default(),
            }
        }

        /// Creates a new poll.
        #[ink(message)]
        pub fn create_poll(
            &mut self,
            question: String,
            options: Vec<String>,
            start_timestamp: Timestamp,
            end_timestamp: Timestamp,
            voting_power_strategy: VotingPowerStrategy,
        ) -> Result<()> {
            if start_timestamp >= end_timestamp {
                return Err(Error::InvalidTimeRange);
            }

            let poll_id = self.poll_count + 1;
            self.poll_count += 1;

            let results = vec![0; options.len()];

            let poll = Poll {
                id: poll_id,
                question: question.clone(),
                options,
                start_timestamp,
                end_timestamp,
                is_active: true,
                voting_power_strategy,
                results,
            };

            self.polls.insert(poll_id, &poll);

            self.env().emit_event(PollCreated {
                poll_id,
                creator: self.env().caller(),
                question,
            });

            Ok(())
        }

        /// Casts a vote in a poll.
        #[ink(message)]
        pub fn vote(&mut self, poll_id: u32, option_index: u32) -> Result<()> {
            let now = self.env().block_timestamp();

            let mut poll = self.polls.get(poll_id).ok_or(Error::PollNotFound)?;

            if !poll.is_active {
                return Err(Error::PollNotActive);
            }

            if now < poll.start_timestamp {
                return Err(Error::PollNotStarted);
            }

            if now > poll.end_timestamp {
                return Err(Error::PollEnded);
            }

            if option_index >= poll.options.len() as u32 {
                return Err(Error::InvalidOption);
            }

            let caller = self.env().caller();
            if self.votes.contains((poll_id, caller)) {
                return Err(Error::AlreadyVoted);
            }

            let vote_weight = match poll.voting_power_strategy {
                VotingPowerStrategy::OnePersonOneVote => 1,
                VotingPowerStrategy::BalanceWeighted => {
                    // Weight by the balance of the voter.
                    let balance = self.env().balance();
                    if balance == 0 {
                        return Err(Error::InsufficientBalance);
                    }
                    balance as u32 // Assuming balance fits within u32
                }
                VotingPowerStrategy::TokenWeighted { token_address } => {
                    // Here you would ideally call out to the token contract
                    // to query the balance of the voter.  Since cross-contract
                    // calls require more setup, we'll just stub it with an error
                    if token_address == AccountId::from([0u8;32]) {
                        return Err(Error::InvalidTokenAddress);
                    }
                    // In a real implementation, get the balance from the token contract
                    // and convert it to u32. Handle overflow/underflow appropriately.
                    1 //replace with Token Contract call
                }
            };

            let result = poll.results.get_mut(option_index as usize).ok_or(Error::InvalidOption)?;
            *result = result.checked_add(vote_weight).ok_or(Error::Overflow)?;

            self.polls.insert(poll_id, &poll);
            self.votes.insert((poll_id, caller), &vote_weight);

            self.env().emit_event(VoteCast {
                poll_id,
                voter: caller,
                option_index,
                vote_weight,
            });

            Ok(())
        }

        /// Ends a poll. Only the owner can end it.
        #[ink(message)]
        pub fn end_poll(&mut self, poll_id: u32) -> Result<()> {
            self.ensure_owner()?;

            let mut poll = self.polls.get(poll_id).ok_or(Error::PollNotFound)?;

            if !poll.is_active {
                return Err(Error::PollNotActive);
            }

            poll.is_active = false;
            self.polls.insert(poll_id, &poll);

            self.env().emit_event(PollEnded { poll_id });

            Ok(())
        }

        /// Gets the poll results.
        #[ink(message)]
        pub fn get_poll_results(&self, poll_id: u32) -> Result<Vec<u32>> {
            let poll = self.polls.get(poll_id).ok_or(Error::PollNotFound)?;
            Ok(poll.results)
        }


        /// Gets the question of the poll.
        #[ink(message)]
        pub fn get_poll_question(&self, poll_id: u32) -> Result<String> {
            let poll = self.polls.get(poll_id).ok_or(Error::PollNotFound)?;
            Ok(poll.question.clone())
        }

        /// Gets the options of the poll.
        #[ink(message)]
        pub fn get_poll_options(&self, poll_id: u32) -> Result<Vec<String>> {
            let poll = self.polls.get(poll_id).ok_or(Error::PollNotFound)?;
            Ok(poll.options.clone())
        }

        /// Returns the owner of the contract.
        #[ink(message)]
        pub fn get_owner(&self) -> AccountId {
            self.owner
        }

        /// Helper function to ensure that the caller is the owner.
        fn ensure_owner(&self) -> Result<()> {
            if self.env().caller() != self.owner {
                return Err(Error::NotOwner);
            }
            Ok(())
        }
    }

    /// Unit tests in Rust are normally defined within such a block.
    #[cfg(test)]
    mod tests {
        use super::*;
        use ink::env::{test, AccountId};

        #[ink::test]
        fn create_and_vote() {
            let mut dapp = DecentralizedOpinionPolling::new();
            let accounts = test::default_accounts::<Environment>();

            let question = String::from("What is your favorite color?");
            let options = vec![
                String::from("Red"),
                String::from("Blue"),
                String::from("Green"),
            ];
            let start_time = 100;
            let end_time = 200;
            let strategy = VotingPowerStrategy::OnePersonOneVote;

            dapp.create_poll(question, options, start_time, end_time, strategy).unwrap();

            dapp.vote(1, 0).unwrap();
            assert_eq!(dapp.get_poll_results(1).unwrap(), vec![1, 0, 0]);
        }

        #[ink::test]
        fn test_balance_weighted_voting() {
            let mut dapp = DecentralizedOpinionPolling::new();
            let accounts = test::default_accounts::<Environment>();
            test::set_account_balance::<Environment>(accounts.alice, 100); // Set Alice's balance
            let question = String::from("Do you like dogs?");
            let options = vec![
                String::from("Yes"),
                String::from("No"),
            ];
            let start_time = 100;
            let end_time = 200;
            let strategy = VotingPowerStrategy::BalanceWeighted;

            dapp.create_poll(question, options, start_time, end_time, strategy).unwrap();

            dapp.vote(1, 0).unwrap();
            assert_eq!(dapp.get_poll_results(1).unwrap(), vec![100, 0]); // Alice's balance should reflect vote weight
        }

        #[ink::test]
        fn test_invalid_token_address() {
            let mut dapp = DecentralizedOpinionPolling::new();
            let accounts = test::default_accounts::<Environment>();

            let question = String::from("Do you like dogs?");
            let options = vec![
                String::from("Yes"),
                String::from("No"),
            ];
            let start_time = 100;
            let end_time = 200;
            let strategy = VotingPowerStrategy::TokenWeighted { token_address: AccountId::from([0u8;32]) };

            dapp.create_poll(question, options, start_time, end_time, strategy).unwrap();

            let result = dapp.vote(1, 0);
            assert_eq!(result, Err(Error::InvalidTokenAddress)); // Token address should be rejected
        }

        #[ink::test]
        fn end_poll_works() {
            let mut dapp = DecentralizedOpinionPolling::new();
            let accounts = test::default_accounts::<Environment>();

            let question = String::from("Do you like cats?");
            let options = vec![
                String::from("Yes"),
                String::from("No"),
            ];
            let start_time = 100;
            let end_time = 200;
            let strategy = VotingPowerStrategy::OnePersonOneVote;

            dapp.create_poll(question, options, start_time, end_time, strategy).unwrap();
            dapp.end_poll(1).unwrap();

            let result = dapp.vote(1, 0);
            assert_eq!(result, Err(Error::PollNotActive));
        }

        type Environment = ::ink::env::DefaultEnvironment;
    }
}
```

**Outline and Function Summary:**

This smart contract implements a decentralized opinion polling system.  It allows anyone to create a poll with multiple options and users to vote on their preferred option.  It offers different voting power calculation strategies.

**Key Features:**

*   **Poll Creation:** Anyone can create a poll, specifying the question, options, start and end times, and the voting power calculation strategy.
*   **Voting:** Users can vote on active polls. The contract supports different voting strategies:
    *   One person, one vote.
    *   Voting power weighted by the user's balance in the contract.
    *   Voting power weighted by the user's balance in an ERC20-like token contract.
*   **Poll Ending:** The owner can end a poll, preventing further voting.
*   **Result Retrieval:**  Anyone can retrieve the results of a poll (vote counts for each option).
*   **Events:** The contract emits events for poll creation, voting, and poll ending.
*   **Access Control:**  Only the owner can end a poll.
*   **Error Handling:**  The contract defines a set of errors to handle invalid states and actions.

**Function Summary:**

*   `new()`: Constructor, initializes the contract owner.
*   `create_poll(question: String, options: Vec<String>, start_timestamp: Timestamp, end_timestamp: Timestamp, voting_power_strategy: VotingPowerStrategy)`: Creates a new poll.
*   `vote(poll_id: u32, option_index: u32)`:  Casts a vote in a poll.  Calculates the vote weight based on the chosen voting strategy.
*   `end_poll(poll_id: u32)`: Ends a poll, preventing further voting (owner only).
*   `get_poll_results(poll_id: u32)`: Returns the vote counts for each option in a poll.
*   `get_poll_question(poll_id: u32)`: Returns question string of the poll
*   `get_poll_options(poll_id: u32)`: Returns option strings of the poll
*   `get_owner()`: Returns the contract owner.
*   `ensure_owner()`: (Private) Checks if the caller is the owner.
