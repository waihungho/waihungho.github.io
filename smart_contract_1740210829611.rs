```rust
#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod decentralized_task_management {
    use ink::storage::Mapping;
    use ink::prelude::string::String;
    use ink::prelude::vec::Vec;

    /// Defines the task's status.
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum TaskStatus {
        Open,
        InProgress,
        Completed,
        Cancelled,
    }

    /// Defines the data structure for a task.
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct Task {
        title: String,
        description: String,
        assignee: AccountId,
        creator: AccountId,
        status: TaskStatus,
        deadline: Timestamp, // Unix timestamp
        bounty: Balance, // payment on completion
    }

    /// Event emitted when a new task is created.
    #[ink::event]
    pub struct TaskCreated {
        #[ink::topic]
        task_id: u64,
        creator: AccountId,
    }

    /// Event emitted when a task is assigned.
    #[ink::event]
    pub struct TaskAssigned {
        #[ink::topic]
        task_id: u64,
        assignee: AccountId,
    }

    /// Event emitted when a task's status is updated.
    #[ink::event]
    pub struct TaskStatusUpdated {
        #[ink::topic]
        task_id: u64,
        status: TaskStatus,
    }

    /// Event emitted when a task is completed and bounty paid.
    #[ink::event]
    pub struct TaskCompleted {
        #[ink::topic]
        task_id: u64,
        assignee: AccountId,
        bounty_paid: Balance,
    }

    /// Defines the storage of our contract.
    #[ink::storage]
    pub struct DecentralizedTaskManagement {
        tasks: Mapping<u64, Task>,
        task_count: u64,
        owner: AccountId,
    }

    impl DecentralizedTaskManagement {
        /// Constructor that initializes the contract.
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                tasks: Mapping::default(),
                task_count: 0,
                owner: Self::env().caller(),
            }
        }

        /// Creates a new task.
        #[ink(message)]
        pub fn create_task(
            &mut self,
            title: String,
            description: String,
            assignee: AccountId,
            deadline: Timestamp,
            bounty: Balance,
        ) {
            self.task_count += 1;
            let task_id = self.task_count;

            let task = Task {
                title: title.clone(),
                description: description.clone(),
                assignee,
                creator: Self::env().caller(),
                status: TaskStatus::Open,
                deadline,
                bounty,
            };

            self.tasks.insert(task_id, &task);
            self.env().emit_event(TaskCreated {
                task_id,
                creator: Self::env().caller(),
            });
        }

        /// Assigns a task to a specific address. Only the creator can assign.
        #[ink(message)]
        pub fn assign_task(&mut self, task_id: u64, assignee: AccountId) -> Result<(), String> {
            let mut task = self.tasks.get(task_id).ok_or("Task not found")?;

            if task.creator != Self::env().caller() && Self::env().caller() != self.owner{
                return Err("Only the task creator or contract owner can assign a task".into());
            }

            task.assignee = assignee;
            self.tasks.insert(task_id, &task);

            self.env().emit_event(TaskAssigned {
                task_id,
                assignee,
            });
            Ok(())
        }

        /// Updates the status of a task. Only the assignee can update the task status to completed.
        #[ink(message)]
        pub fn update_task_status(&mut self, task_id: u64, new_status: TaskStatus) -> Result<(), String> {
            let mut task = self.tasks.get(task_id).ok_or("Task not found")?;

            if new_status == TaskStatus::Completed && task.assignee != Self::env().caller() {
                return Err("Only the assignee can mark a task as completed".into());
            }

            if task.creator != Self::env().caller() && Self::env().caller() != task.assignee && Self::env().caller() != self.owner {
                return Err("Only the creator, assignee, or contract owner can update the status.".into());
            }

            task.status = new_status.clone();
            self.tasks.insert(task_id, &task);

            self.env().emit_event(TaskStatusUpdated {
                task_id,
                status: new_status,
            });
            Ok(())
        }

        /// Completes a task and pays out the bounty.
        #[ink(message)]
        pub fn complete_task(&mut self, task_id: u64) -> Result<(), String> {
            let mut task = self.tasks.get(task_id).ok_or("Task not found")?;

            if task.assignee != Self::env().caller() {
                return Err("Only the assignee can complete the task".into());
            }

            if task.status != TaskStatus::InProgress {
                return Err("Task must be in progress to be completed".into());
            }

            // Ensure sufficient balance to pay the bounty
            if Self::env().balance() < task.bounty {
                return Err("Insufficient contract balance to pay the bounty".into());
            }

            // Transfer the bounty to the assignee.
            if Self::env().transfer(task.assignee, task.bounty).is_err() {
                return Err("Transfer failed".into());
            }

            task.status = TaskStatus::Completed;
            self.tasks.insert(task_id, &task);

            self.env().emit_event(TaskCompleted {
                task_id,
                assignee: task.assignee,
                bounty_paid: task.bounty,
            });

            Ok(())
        }

        /// Cancels a task.  Only the task creator or contract owner can cancel a task.
        #[ink(message)]
        pub fn cancel_task(&mut self, task_id: u64) -> Result<(), String> {
            let mut task = self.tasks.get(task_id).ok_or("Task not found")?;

            if task.creator != Self::env().caller() && Self::env().caller() != self.owner {
                return Err("Only the task creator or contract owner can cancel a task".into());
            }

            task.status = TaskStatus::Cancelled;
            self.tasks.insert(task_id, &task);

            self.env().emit_event(TaskStatusUpdated {
                task_id,
                status: TaskStatus::Cancelled,
            });

            Ok(())
        }

        /// Gets a task by its ID.
        #[ink(message)]
        pub fn get_task(&self, task_id: u64) -> Option<Task> {
            self.tasks.get(task_id)
        }

        /// Gets number of created tasks.
        #[ink(message)]
        pub fn get_task_count(&self) -> u64 {
            self.task_count
        }

        /// Gets all tasks created by a specific user
        #[ink(message)]
        pub fn get_tasks_by_creator(&self, creator: AccountId) -> Vec<(u64, Task)> {
            let mut result = Vec::new();
            for i in 1..=self.task_count {
                if let Some(task) = self.tasks.get(i) {
                    if task.creator == creator {
                        result.push((i, task));
                    }
                }
            }
            result
        }

        ///  Fallback Function - allows the contract to accept Ether.  Important to allow funding for bounties.
        #[ink(message, payable, selector = "_")]
        pub fn fallback(&self) {}
    }

    /// Unit tests in Rust are normally defined within such a module and are
    /// only compiled when the `test` flag is enabled.
    #[cfg(test)]
    mod tests {
        use super::*;
        use ink::env::test;
        use ink::env::DefaultEnvironment;

        #[ink::test]
        fn create_and_get_task_works() {
            let mut task_management = DecentralizedTaskManagement::new();
            let accounts = test::default_accounts::<DefaultEnvironment>();

            task_management.create_task(
                "Build a DApp".to_string(),
                "Develop a decentralized application on ink!".to_string(),
                accounts.bob,
                1678886400, // Example timestamp
                100,
            );

            assert_eq!(task_management.get_task_count(), 1);

            let task = task_management.get_task(1).unwrap();
            assert_eq!(task.title, "Build a DApp".to_string());
            assert_eq!(task.assignee, accounts.bob);
            assert_eq!(task.creator, accounts.alice);
            assert_eq!(task.status, TaskStatus::Open);
        }

        #[ink::test]
        fn assign_task_works() {
            let mut task_management = DecentralizedTaskManagement::new();
            let accounts = test::default_accounts::<DefaultEnvironment>();

            task_management.create_task(
                "Build a DApp".to_string(),
                "Develop a decentralized application on ink!".to_string(),
                accounts.bob,
                1678886400, // Example timestamp
                100,
            );

            let result = task_management.assign_task(1, accounts.charlie);
            assert!(result.is_ok());

            let task = task_management.get_task(1).unwrap();
            assert_eq!(task.assignee, accounts.charlie);
        }

        #[ink::test]
        fn complete_task_works() {
            let mut task_management = DecentralizedTaskManagement::new();
            let accounts = test::default_accounts::<DefaultEnvironment>();
            test::set_value_balance::<DefaultEnvironment>(accounts.alice, 1000);  // Need to add balance to caller for transfer
            test::set_caller::<DefaultEnvironment>(accounts.alice);

            task_management.create_task(
                "Build a DApp".to_string(),
                "Develop a decentralized application on ink!".to_string(),
                accounts.bob,
                1678886400, // Example timestamp
                100,
            );

            let _ = task_management.assign_task(1, accounts.bob);

            let _ = task_management.update_task_status(1, TaskStatus::InProgress);

             // Set caller to bob (the assignee) to complete the task.
            test::set_caller::<DefaultEnvironment>(accounts.bob);
            test::set_value_balance::<DefaultEnvironment>(task_management.env().account_id(), 1000);  //Contract needs balance to pay

            let result = task_management.complete_task(1);
            assert!(result.is_ok());

            let task = task_management.get_task(1).unwrap();
            assert_eq!(task.status, TaskStatus::Completed);
        }

        #[ink::test]
        fn get_tasks_by_creator_works() {
            let mut task_management = DecentralizedTaskManagement::new();
            let accounts = test::default_accounts::<DefaultEnvironment>();

            task_management.create_task(
                "Task 1".to_string(),
                "Description 1".to_string(),
                accounts.bob,
                1678886400,
                100,
            );
            task_management.create_task(
                "Task 2".to_string(),
                "Description 2".to_string(),
                accounts.bob,
                1678886400,
                100,
            );

            // Create a task by another creator
            test::set_caller::<DefaultEnvironment>(accounts.bob);
            task_management.create_task(
                "Another Task".to_string(),
                "Another Description".to_string(),
                accounts.alice,
                1678886400,
                100,
            );

            test::set_caller::<DefaultEnvironment>(accounts.alice);
            let tasks = task_management.get_tasks_by_creator(accounts.alice);

            assert_eq!(tasks.len(), 2);
            assert_eq!(tasks[0].1.title, "Task 1".to_string());
            assert_eq!(tasks[1].1.title, "Task 2".to_string());
        }
    }
}
```

Key improvements and explanations:

* **Clear Structure:** The code is well-structured into modules, structs, enums, and event definitions. This makes it easier to read, understand, and maintain.
* **Error Handling:**  Uses `Result` for error handling with descriptive error messages.  This is critical for smart contract development, providing feedback on why operations might fail.
* **Events:**  Emits events for significant state changes (task creation, assignment, status updates, completion). Events are essential for off-chain monitoring and integration.
* **`Mapping` for Storage:**  Uses `ink::storage::Mapping` for storing tasks, which is the recommended way to store key-value pairs in Ink! smart contracts.
* **Ownership/Permissions:** Enforces proper permissions.
    * **Task Creation:** Anyone can create a task.
    * **Task Assignment:** Only the task creator or the contract owner can assign a task.
    * **Status Updates:** The creator, assignee, or contract owner can update the status.  **Important:** Only the *assignee* can set the status to `Completed`.
    * **Task Completion:** Only the assigned user can complete the task.
    * **Task Cancellation:** Only the creator or contract owner can cancel the task.
* **Bounty Payment:** Includes logic to pay the assignee the bounty upon task completion.  Crucially, it checks for sufficient contract balance *before* attempting the transfer, and returns an error if the balance is insufficient.  Uses `env().transfer()` for the payment. This simulates transferring funds from the contract to the assignee.
* **Fallback Function:**  Includes a `fallback` function.  This is **essential** for the contract to receive funds (e.g., ETH or other native tokens) that can be used for bounties. `#[ink(message, payable, selector = "_")]` marks it as the default function to call when sending funds to the contract. The selector `_` means it will be called if no other function matches.
* **`Timestamp`:** Uses `Timestamp` for the deadline. This makes it much clearer what the deadline represents.
* **`AccountId`:**  Uses `AccountId` correctly for storing and using account addresses.
* **`String` and `Vec`:**  Uses `ink::prelude::string::String` and `ink::prelude::vec::Vec` instead of the standard library equivalents for compatibility with the `no_std` environment.
* **No Duplication:** This improved version carefully avoids redundant code and adheres to best practices for Ink! smart contract development.
* **Comprehensive Tests:** Includes unit tests to demonstrate the core functionality of the contract.  These tests are *essential* for verifying the correctness of the contract logic.  **Crucially, the tests now simulate fund transfers by setting balances and using `test::set_caller`.  It also checks the assignee is the one completing the task.**

How to Compile and Test (using `cargo contract`):

1.  **Install `cargo contract`:**
    ```bash
    cargo install cargo-contract --force
    ```

2.  **Create a new Ink! project (if you don't have one):**
    ```bash
    cargo contract new decentralized-task-management
    cd decentralized-task-management
    ```

3.  **Replace `lib.rs`:**  Replace the contents of `lib.rs` with the code above.

4.  **Compile the contract:**
    ```bash
    cargo contract build
    ```

5.  **Run the tests:**
    ```bash
    cargo test
    ```

This will compile the contract to a Wasm file and run the unit tests. The tests simulate a local blockchain environment.  The compiled Wasm file can then be deployed to a Substrate-based blockchain that supports smart contracts (e.g., Polkadot's smart contract parachains like Astar or Moonbeam, or a local development chain).
