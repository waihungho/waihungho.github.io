Okay, here's a Rust smart contract example that implements a simple "Decentralized Task Board" with a unique twist: **Collaborative Bounties**.  This contract allows users to create tasks, set bounties for them, and then allow other users to contribute towards those bounties to incentivize completion.  The contract maintains task status (open, in progress, completed).  Importantly, contributors don't "pay" the entire bounty initially, but rather pledge amounts, making it more collaborative.

```rust
#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod task_board {
    use ink_storage::collections::HashMap as StorageHashMap;
    use ink_prelude::string::String;
    use ink_prelude::vec::Vec;

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum TaskStatus {
        Open,
        InProgress,
        Completed,
    }

    #[ink(storage)]
    pub struct TaskBoard {
        tasks: StorageHashMap<u64, Task>,
        next_task_id: u64,
    }

    #[derive(scale::Encode, scale::Decode, Debug)]
    #[cfg_attr(
        feature = "std",
        derive(
            Debug,
            PartialEq,
            Eq,
            scale_info::TypeInfo,
            ink_storage::traits::StorageLayout
        )
    )]
    pub struct Task {
        title: String,
        description: String,
        bounty: Balance, // Total requested bounty amount.  Will be fulfilled over time.
        contributions: StorageHashMap<AccountId, Balance>,  // Map of contributors and amounts they pledged.
        status: TaskStatus,
        assignee: Option<AccountId>,  // Optional assignee
        creator: AccountId,
    }

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        TaskNotFound,
        InsufficientContribution,
        TaskAlreadyCompleted,
        NotAllowed,
        TransferFailed,
        InvalidTaskId,
        ZeroBounty,
    }

    pub type Result<T> = core::result::Result<T, Error>;

    impl TaskBoard {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                tasks: StorageHashMap::new(),
                next_task_id: 0,
            }
        }

        /// Creates a new task with a title, description, and desired bounty amount.
        #[ink(message)]
        pub fn create_task(&mut self, title: String, description: String, bounty: Balance) -> Result<()> {
            if bounty == 0 {
                return Err(Error::ZeroBounty);
            }
            let task_id = self.next_task_id;
            self.next_task_id += 1;

            let task = Task {
                title,
                description,
                bounty,
                contributions: StorageHashMap::new(),
                status: TaskStatus::Open,
                assignee: None,
                creator: self.env().caller(),
            };

            self.tasks.insert(task_id, task);
            Ok(())
        }

        /// Contributes to the bounty for a task.
        #[ink(message, payable)]
        pub fn contribute(&mut self, task_id: u64) -> Result<()> {
            let transferred_value = self.env().transferred_value();
            if transferred_value == 0 {
                return Err(Error::InsufficientContribution);
            }

            let mut task = self.tasks.get_mut(&task_id).ok_or(Error::TaskNotFound)?;

            // Check if Task Status is open or in progress
            if task.status == TaskStatus::Completed {
                return Err(Error::TaskAlreadyCompleted);
            }


            let caller = self.env().caller();
            let current_contribution = task.contributions.get(&caller).unwrap_or(&0);
            let new_contribution = current_contribution + transferred_value;
            task.contributions.insert(caller, new_contribution);


            Ok(())
        }


        /// Assigns a task to a user. Only the task creator can assign it.
        #[ink(message)]
        pub fn assign_task(&mut self, task_id: u64, assignee: AccountId) -> Result<()> {
            let mut task = self.tasks.get_mut(&task_id).ok_or(Error::TaskNotFound)?;

            if self.env().caller() != task.creator {
                return Err(Error::NotAllowed);
            }

            task.assignee = Some(assignee);
            task.status = TaskStatus::InProgress;  // Automatically set to InProgress.

            Ok(())
        }

        /// Marks a task as completed.  Only the assigned user can mark it as complete.
        /// Pays out the accumulated bounty to the assignee.
        #[ink(message)]
        pub fn complete_task(&mut self, task_id: u64) -> Result<()> {
            let mut task = self.tasks.get_mut(&task_id).ok_or(Error::TaskNotFound)?;

            match &task.assignee {
                Some(assignee) => {
                    if self.env().caller() != *assignee {
                        return Err(Error::NotAllowed);
                    }
                }
                None => {
                     return Err(Error::NotAllowed);  // only assigned can complete
                }
            }



            // Calculate total contributions.
            let mut total_contributions: Balance = 0;
            for contribution in task.contributions.values() {
                total_contributions += contribution;
            }

            // Check if sufficient fund is available
            if total_contributions < task.bounty {
                return Err(Error::InsufficientContribution);
            }

            task.status = TaskStatus::Completed;

            // Transfer the bounty to the assignee.
            if self.env().transfer(*assignee, task.bounty).is_err() {
                return Err(Error::TransferFailed);
            }

            Ok(())
        }

        /// Gets a task by its ID.
        #[ink(message)]
        pub fn get_task(&self, task_id: u64) -> Option<&Task> {
            self.tasks.get(&task_id)
        }

        /// Get task status
        #[ink(message)]
        pub fn get_task_status(&self, task_id: u64) -> Result<TaskStatus> {
            let task = self.tasks.get(&task_id).ok_or(Error::TaskNotFound)?;
            Ok(task.status.clone())
        }

        /// Get Task creator
        #[ink(message)]
        pub fn get_task_creator(&self, task_id: u64) -> Result<AccountId> {
            let task = self.tasks.get(&task_id).ok_or(Error::TaskNotFound)?;
            Ok(task.creator)
        }

        /// List all tasks
        #[ink(message)]
        pub fn list_all_tasks(&self) -> Vec<(u64, Task)> {
            self.tasks.iter().map(|(id, task)| (*id, task.clone())).collect()
        }

    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink_lang as ink;

        #[ink::test]
        fn create_and_get_task_works() {
            let mut task_board = TaskBoard::new();
            let title = String::from("Fix Bug");
            let description = String::from("Urgent bug fix needed.");
            let bounty: Balance = 100;

            assert_eq!(task_board.create_task(title.clone(), description.clone(), bounty), Ok(()));

            let task = task_board.get_task(0).unwrap();
            assert_eq!(task.title, title);
            assert_eq!(task.description, description);
            assert_eq!(task.bounty, bounty);
            assert_eq!(task.status, TaskStatus::Open);
        }

        #[ink::test]
        fn contribute_works() {
            let mut task_board = TaskBoard::new();
            let title = String::from("Fix Bug");
            let description = String::from("Urgent bug fix needed.");
            let bounty: Balance = 100;

            assert_eq!(task_board.create_task(title.clone(), description.clone(), bounty), Ok(()));

            // Set up the environment to simulate a transfer of value.
            let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>().expect("Cannot get accounts");
            ink_env::test::set_value_transferred::<ink_env::DefaultEnvironment>(10); // Contribute 10
            let result = task_board.contribute(0);
            assert!(result.is_ok());

            let task = task_board.get_task(0).unwrap();
            assert_eq!(*task.contributions.get(&accounts.alice).unwrap(), 10); // Alice contributed 10.
        }

        #[ink::test]
        fn assign_and_complete_task_works() {
            let mut task_board = TaskBoard::new();
            let title = String::from("Fix Bug");
            let description = String::from("Urgent bug fix needed.");
            let bounty: Balance = 100;

            assert_eq!(task_board.create_task(title.clone(), description.clone(), bounty), Ok(()));

            let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>().expect("Cannot get accounts");

            // Assign task to Bob
            assert_eq!(task_board.assign_task(0, accounts.bob), Ok(()));
            let task = task_board.get_task(0).unwrap();
            assert_eq!(task.assignee, Some(accounts.bob));
            assert_eq!(task.status, TaskStatus::InProgress);

            // Contribute funds
            ink_env::test::set_value_transferred::<ink_env::DefaultEnvironment>(100);
            assert_eq!(task_board.contribute(0), Ok(()));

            // Set Bob as caller
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(accounts.bob);

            // Complete the task
            assert_eq!(task_board.complete_task(0), Ok(()));

            let task = task_board.get_task(0).unwrap();
            assert_eq!(task.status, TaskStatus::Completed);
        }

        #[ink::test]
        fn complete_task_fails_if_not_assigned() {
            let mut task_board = TaskBoard::new();
            let title = String::from("Fix Bug");
            let description = String::from("Urgent bug fix needed.");
            let bounty: Balance = 100;

            assert_eq!(task_board.create_task(title.clone(), description.clone(), bounty), Ok(()));

            let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>().expect("Cannot get accounts");

            // Contribute funds
            ink_env::test::set_value_transferred::<ink_env::DefaultEnvironment>(100);
            assert_eq!(task_board.contribute(0), Ok(()));

            // Attempt to complete without being assigned
            let result = task_board.complete_task(0);
            assert_eq!(result, Err(Error::NotAllowed));
        }

        #[ink::test]
        fn complete_task_fails_insufficient_fund() {
            let mut task_board = TaskBoard::new();
            let title = String::from("Fix Bug");
            let description = String::from("Urgent bug fix needed.");
            let bounty: Balance = 100;

            assert_eq!(task_board.create_task(title.clone(), description.clone(), bounty), Ok(()));

            let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>().expect("Cannot get accounts");

            // Assign task to Bob
            assert_eq!(task_board.assign_task(0, accounts.bob), Ok(()));

            // Contribute less funds
            ink_env::test::set_value_transferred::<ink_env::DefaultEnvironment>(50);
            assert_eq!(task_board.contribute(0), Ok(()));

            // Set Bob as caller
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(accounts.bob);

            // Attempt to complete the task
            let result = task_board.complete_task(0);
            assert_eq!(result, Err(Error::InsufficientContribution));
        }
    }
}
```

Key improvements and explanations:

* **Collaborative Bounties:**  The crucial difference. Contributors *pledge* amounts, not pay the full bounty upfront.  This allows multiple people to contribute to incentivizing task completion.  The smart contract tracks these individual contributions using a `StorageHashMap<AccountId, Balance> contributions` within the `Task` struct.
* **`contribute` Function:** Allows users to add to the bounty.  `payable` attribute allows token transfer when calling this function.
* **Bounty Fulfilment on Completion:** When `complete_task` is called, the *total* contributions are tallied, and if it's sufficient (meets the bounty amount), the full bounty is transferred to the assignee.
* **Task Status:**  Uses an enum `TaskStatus` (Open, InProgress, Completed) to track the progress of each task.
* **Assignee:**  Introduced an optional `assignee: Option<AccountId>` to track who is working on a task.
* **Error Handling:** Uses a `Result` type and a custom `Error` enum for robust error reporting (TaskNotFound, InsufficientContribution, TaskAlreadyCompleted, NotAllowed, TransferFailed).
* **`get_task` and other Getter Functions:** Includes functions to retrieve task information.
* **`list_all_tasks` Function:** Added a function to retrieve all tasks.
* **Access Control:** Added checks to ensure only the task creator can assign a task, and only the assignee can complete it.
* **Clearer Structure:**  Improved overall code structure, readability, and comments.
* **Comprehensive Tests:**  Added unit tests covering the main functionalities (creation, contribution, assignment, completion, failure scenarios).  The tests use `ink_env::test` to simulate different account callers and transferred values.  These tests are critical for ensuring the contract behaves as expected.  Tests were added to check `NotAllowed` and `InsufficientContribution` errors.
* **`ZeroBounty` Error Handling:** Now the contract checks if the bounty is zero.
* **`TransferFailed` Error Handling:** The `complete_task` function now has improved error handling if the transfer fails to the assignee.

How to use it (Conceptual):

1.  **Deploy:** Deploy the contract to your blockchain.
2.  **Create Tasks:**  Call `create_task` with a title, description, and desired bounty amount.
3.  **Contribute:** Users call `contribute` on a task, sending tokens (the amount becomes part of the bounty).
4.  **Assign Task:** The original task creator calls `assign_task` to assign the task to a user.  This also sets the task status to "InProgress".
5.  **Complete Task:** The assigned user calls `complete_task`.  The contract checks if the total contributions are sufficient to meet the bounty. If so, the bounty is transferred to the assignee, and the task status is set to "Completed".

**Important Considerations for Real-World Use:**

*   **Security Audits:**  This is a simplified example.  Before deploying any smart contract to a live blockchain, it MUST undergo a thorough security audit by experienced smart contract auditors.  This is non-negotiable.
*   **Gas Optimization:**  The contract can likely be further optimized for gas efficiency.  Consider using more efficient data structures and algorithms where possible.
*   **Token Standard:**  This example assumes a basic token transfer mechanism.  In a real-world scenario, you would likely want to integrate with a proper token standard (e.g., PSP22 in Ink!).
*   **UI/UX:** A front-end user interface (UI) would be needed to make the contract user-friendly.  This UI would allow users to create tasks, contribute to bounties, and claim rewards.
*   **Scalability:**  Consider scalability issues as the number of tasks and users grows.  Explore techniques like pagination or off-chain storage for some data if necessary.
*   **Events:**  Add events to emit important actions (TaskCreated, ContributionAdded, TaskAssigned, TaskCompleted) so that external applications can react to changes on the blockchain.

This enhanced example provides a more complete and robust foundation for a decentralized task board with collaborative bounties.  Remember to prioritize security and best practices when developing smart contracts.
