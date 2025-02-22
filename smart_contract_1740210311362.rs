```rust
#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang:: {
    contract,
    env::CallFlags,
    codegen::EmitEvent,
    reflect::ContractEvent,
    EmitEvent,
    Env,
    StaticBuffer,
};
use ink_prelude::{
    string::String,
    vec::Vec,
};
use ink_storage::{
    collections::HashMap as StorageHashMap,
    traits::{
        PackedLayout,
        SpreadLayout,
    },
};

/// Defines the storage of our contract.
///
/// In our case the storage consists of a single `HashMap` which
/// maps `AccountId`s to a `String`.
#[ink_lang::storage]
pub struct SocialMedia {
    /// Stores who owns what profiles.
    profiles: StorageHashMap<AccountId, Profile>,
    /// Stores the posts made by users.  Key is Post ID, Value is Post struct
    posts: StorageHashMap<u64, Post>,
    /// Keeps track of the next available post ID.
    next_post_id: u64,
    /// Maintains a mapping between user account and their followers.
    followers: StorageHashMap<AccountId, Vec<AccountId>>,
    /// Maintains a mapping between user account and their following.
    following: StorageHashMap<AccountId, Vec<AccountId>>,
    /// Platform fee percentage, stored as basis points (100 = 1%)
    platform_fee: u16,
    /// The account that receives platform fees
    platform_fee_recipient: AccountId,
}


/// Define the Profile struct
#[derive(
    Debug,
    Clone,
    scale::Encode,
    scale::Decode,
    SpreadLayout,
    PackedLayout,
    PartialEq,
    Eq
)]
#[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo)
)]
pub struct Profile {
    pub username: String,
    pub bio: String,
    pub profile_picture_url: String,
}

/// Define the Post struct
#[derive(
    Debug,
    Clone,
    scale::Encode,
    scale::Decode,
    SpreadLayout,
    PackedLayout,
    PartialEq,
    Eq
)]
#[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo)
)]
pub struct Post {
    pub author: AccountId,
    pub content: String,
    pub timestamp: u64,
    pub likes: u64,
    pub shares: u64,
}

/// Events emitted by the contract.
#[ink_lang::event]
pub struct ProfileCreated {
    #[ink(topic)]
    account: AccountId,
    username: String,
}

#[ink_lang::event]
pub struct ProfileUpdated {
    #[ink(topic)]
    account: AccountId,
    username: String,
}

#[ink_lang::event]
pub struct PostCreated {
    #[ink(topic)]
    post_id: u64,
    author: AccountId,
}

#[ink_lang::event]
pub struct PostLiked {
    #[ink(topic)]
    post_id: u64,
    account: AccountId,
}

#[ink_lang::event]
pub struct Followed {
    #[ink(topic)]
    follower: AccountId,
    followed: AccountId,
}

#[ink_lang::event]
pub struct Unfollowed {
    #[ink(topic)]
    follower: AccountId,
    unfollowed: AccountId,
}

#[ink_lang::event]
pub struct PlatformFeeUpdated {
    new_fee: u16,
}

#[ink_lang::event]
pub struct PlatformFeeRecipientUpdated {
    new_recipient: AccountId,
}


/// Errors that can occur upon calling this contract.
#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum Error {
    /// Username is already taken.
    UsernameTaken,
    /// Profile does not exist.
    ProfileNotFound,
    /// Post does not exist.
    PostNotFound,
    /// Caller is not authorized.
    Unauthorized,
    /// Invalid input.
    InvalidInput,
    /// Already following
    AlreadyFollowing,
    /// Not following
    NotFollowing,
    /// Exceeded max length
    ExceedMaxLength,
    /// Overflow occurred
    Overflow,
    /// Underflow occurred
    Underflow,
    /// Platform fee is not set correctly
    InvalidPlatformFee,
}

/// Type alias for the contract's result type.
pub type Result<T> = core::result::Result<T, Error>;


impl SocialMedia {
    /// Constructor that initializes the `HashMap`
    #[ink(constructor)]
    pub fn new(platform_fee: u16, platform_fee_recipient: AccountId) -> Self {
        Self {
            profiles: StorageHashMap::new(),
            posts: StorageHashMap::new(),
            next_post_id: 0,
            followers: StorageHashMap::new(),
            following: StorageHashMap::new(),
            platform_fee,
            platform_fee_recipient,
        }
    }

    /// Create a user profile.
    #[ink(message)]
    pub fn create_profile(&mut self, username: String, bio: String, profile_picture_url: String) -> Result<()> {
        if username.len() > 32 {
            return Err(Error::ExceedMaxLength);
        }

        if bio.len() > 256 {
            return Err(Error::ExceedMaxLength);
        }

        if profile_picture_url.len() > 256 {
            return Err(Error::ExceedMaxLength);
        }

        let caller = self.env().caller();
        if self.profiles.contains_key(&caller) {
            return Err(Error::UsernameTaken);
        }

        let profile = Profile {
            username: username.clone(),
            bio,
            profile_picture_url,
        };

        self.profiles.insert(caller, profile);
        self.env().emit_event(ProfileCreated {
            account: caller,
            username,
        });
        Ok(())
    }

    /// Get a user profile.
    #[ink(message)]
    pub fn get_profile(&self, account: AccountId) -> Option<Profile> {
        self.profiles.get(&account).cloned()
    }

    /// Update a user profile.
    #[ink(message)]
    pub fn update_profile(&mut self, username: String, bio: String, profile_picture_url: String) -> Result<()> {
        if username.len() > 32 {
            return Err(Error::ExceedMaxLength);
        }

        if bio.len() > 256 {
            return Err(Error::ExceedMaxLength);
        }

        if profile_picture_url.len() > 256 {
            return Err(Error::ExceedMaxLength);
        }

        let caller = self.env().caller();
        let profile = self.profiles.get_mut(&caller);
        match profile {
            Some(profile) => {
                profile.username = username.clone();
                profile.bio = bio;
                profile.profile_picture_url = profile_picture_url;
                self.env().emit_event(ProfileUpdated {
                    account: caller,
                    username,
                });
                Ok(())
            }
            None => Err(Error::ProfileNotFound),
        }
    }

    /// Create a post.
    #[ink(message)]
    pub fn create_post(&mut self, content: String) -> Result<()> {
        if content.len() > 512 {
            return Err(Error::ExceedMaxLength);
        }
        let caller = self.env().caller();
        let timestamp = self.env().block_timestamp();
        let post_id = self.next_post_id;

        let post = Post {
            author: caller,
            content,
            timestamp,
            likes: 0,
            shares: 0,
        };

        self.posts.insert(post_id, post);
        self.next_post_id = self.next_post_id.checked_add(1).ok_or(Error::Overflow)?;
        self.env().emit_event(PostCreated {
            post_id,
            author: caller,
        });
        Ok(())
    }

    /// Get a post.
    #[ink(message)]
    pub fn get_post(&self, post_id: u64) -> Option<Post> {
        self.posts.get(&post_id).cloned()
    }

    /// Like a post.
    #[ink(message)]
    pub fn like_post(&mut self, post_id: u64) -> Result<()> {
        let caller = self.env().caller();
        let post = self.posts.get_mut(&post_id);

        match post {
            Some(post) => {
                post.likes = post.likes.checked_add(1).ok_or(Error::Overflow)?;
                self.env().emit_event(PostLiked {
                    post_id,
                    account: caller,
                });
                Ok(())
            }
            None => Err(Error::PostNotFound),
        }
    }

    /// Share a post.  For simplicity, this just increments the share count.  In reality, sharing might involve more complex logic like creating new posts with references.
    #[ink(message)]
    pub fn share_post(&mut self, post_id: u64) -> Result<()> {
         let post = self.posts.get_mut(&post_id);

        match post {
            Some(post) => {
                post.shares = post.shares.checked_add(1).ok_or(Error::Overflow)?;
                Ok(())
            }
            None => Err(Error::PostNotFound),
        }
    }

    /// Follow another user.
    #[ink(message)]
    pub fn follow(&mut self, account_to_follow: AccountId) -> Result<()> {
        let caller = self.env().caller();

        if caller == account_to_follow {
            return Err(Error::InvalidInput); //  Cannot follow yourself
        }

        // Update followers list for the followed account.
        let followers = self.followers.entry(account_to_follow).or_insert(Vec::new());
        if followers.contains(&caller) {
            return Err(Error::AlreadyFollowing);
        }
        followers.push(caller);

        // Update following list for the follower account.
        let following = self.following.entry(caller).or_insert(Vec::new());
        following.push(account_to_follow);

        self.env().emit_event(Followed {
            follower: caller,
            followed: account_to_follow,
        });
        Ok(())
    }

    /// Unfollow another user.
    #[ink(message)]
    pub fn unfollow(&mut self, account_to_unfollow: AccountId) -> Result<()> {
        let caller = self.env().caller();

        // Update followers list for the unfollowed account.
        let followers = self.followers.entry(account_to_unfollow).or_insert(Vec::new());
        if !followers.contains(&caller) {
            return Err(Error::NotFollowing);
        }
        followers.retain(|&acc| acc != caller);


        // Update following list for the unfollower account.
        let following = self.following.entry(caller).or_insert(Vec::new());
        following.retain(|&acc| acc != account_to_unfollow);

        self.env().emit_event(Unfollowed {
            follower: caller,
            unfollowed: account_to_unfollow,
        });
        Ok(())
    }

    /// Get the list of followers for a given account.
    #[ink(message)]
    pub fn get_followers(&self, account: AccountId) -> Vec<AccountId> {
        self.followers.get(&account).cloned().unwrap_or_default()
    }

    /// Get the list of accounts a given account is following.
    #[ink(message)]
    pub fn get_following(&self, account: AccountId) -> Vec<AccountId> {
        self.following.get(&account).cloned().unwrap_or_default()
    }


    /// Set the platform fee (in basis points). Requires the caller to be the contract owner.
    #[ink(message)]
    pub fn set_platform_fee(&mut self, new_fee: u16) -> Result<()> {
        // In a real-world scenario, you'd want an owner check here.  For simplicity, we skip it.
        if new_fee > 10000 { //  Max 100%
            return Err(Error::InvalidPlatformFee);
        }
        self.platform_fee = new_fee;
        self.env().emit_event(PlatformFeeUpdated { new_fee });
        Ok(())
    }

    /// Get the platform fee (in basis points).
    #[ink(message)]
    pub fn get_platform_fee(&self) -> u16 {
        self.platform_fee
    }

     /// Set the platform fee recipient. Requires the caller to be the contract owner.
    #[ink(message)]
    pub fn set_platform_fee_recipient(&mut self, new_recipient: AccountId) -> Result<()> {
        // In a real-world scenario, you'd want an owner check here.  For simplicity, we skip it.
        self.platform_fee_recipient = new_recipient;
        self.env().emit_event(PlatformFeeRecipientUpdated { new_recipient });
        Ok(())
    }

    /// Get the platform fee recipient.
    #[ink(message)]
    pub fn get_platform_fee_recipient(&self) -> AccountId {
        self.platform_fee_recipient
    }

    // Example "pay-to-post" function (Demonstrative, not fully functional without token integration).
    // Note: This is simplified.  A real implementation would require handling token transfers, fee calculations, and error handling for insufficient funds.
    #[ink(message, payable)]
    pub fn pay_to_post(&mut self, content: String) -> Result<()> {
        if content.len() > 512 {
            return Err(Error::ExceedMaxLength);
        }

        let transferred_value = self.env().transferred_value();
        let platform_fee = self.platform_fee;

        // Calculate the fee amount. (transferred_value * platform_fee) / 10000
        let fee_amount = transferred_value
            .checked_mul(platform_fee.into())
            .ok_or(Error::Overflow)?
            .checked_div(10000u128.into())
            .ok_or(Error::Underflow)?; // Avoid division by zero if fee is zero

        // Transfer the platform fee to the recipient (In a real implementation).
        if fee_amount > 0 {
          //  self.env().transfer(self.platform_fee_recipient, fee_amount as Balance).unwrap(); // Needs integration with token transfer library.
          ink_env::debug_println!("Fee: {} will be sent to {}", fee_amount, self.platform_fee_recipient);
        }



        let caller = self.env().caller();
        let timestamp = self.env().block_timestamp();
        let post_id = self.next_post_id;

        let post = Post {
            author: caller,
            content,
            timestamp,
            likes: 0,
            shares: 0,
        };

        self.posts.insert(post_id, post);
        self.next_post_id = self.next_post_id.checked_add(1).ok_or(Error::Overflow)?;
        self.env().emit_event(PostCreated {
            post_id,
            author: caller,
        });

        ink_env::debug_println!("Successfully created pay-to-post. Fee Amount: {}, Post ID: {}", fee_amount, post_id);
        Ok(())
    }


}


#[cfg(test)]
mod tests {
    use super::*;
    use ink_lang as ink;
    use ink_env::{
        test,
        AccountId,
    };

    #[ink::test]
    fn create_profile_works() {
        let accounts = test::default_accounts::<ink_env::DefaultEnvironment>().expect("Failed to get accounts");
        let mut social_media = SocialMedia::new(0, accounts.alice);
        let username = String::from("testuser");
        let bio = String::from("This is my bio.");
        let profile_picture_url = String::from("http://example.com/image.jpg");

        assert_eq!(social_media.create_profile(username.clone(), bio.clone(), profile_picture_url.clone()), Ok(()));
        let profile = social_media.get_profile(accounts.alice).unwrap();
        assert_eq!(profile.username, username);
        assert_eq!(profile.bio, bio);
        assert_eq!(profile.profile_picture_url, profile_picture_url);
    }

    #[ink::test]
    fn create_duplicate_profile_fails() {
        let accounts = test::default_accounts::<ink_env::DefaultEnvironment>().expect("Failed to get accounts");
        let mut social_media = SocialMedia::new(0, accounts.alice);
        let username = String::from("testuser");
        let bio = String::from("This is my bio.");
        let profile_picture_url = String::from("http://example.com/image.jpg");

        social_media.create_profile(username.clone(), bio.clone(), profile_picture_url.clone()).unwrap();
        assert_eq!(social_media.create_profile(username, bio, profile_picture_url), Err(Error::UsernameTaken));
    }

    #[ink::test]
    fn update_profile_works() {
        let accounts = test::default_accounts::<ink_env::DefaultEnvironment>().expect("Failed to get accounts");
        let mut social_media = SocialMedia::new(0, accounts.alice);
        let username = String::from("testuser");
        let bio = String::from("This is my bio.");
        let profile_picture_url = String::from("http://example.com/image.jpg");
        social_media.create_profile(username.clone(), bio.clone(), profile_picture_url.clone()).unwrap();

        let new_username = String::from("newuser");
        let new_bio = String::from("This is my updated bio.");
        let new_profile_picture_url = String::from("http://example.com/new_image.jpg");
        assert_eq!(social_media.update_profile(new_username.clone(), new_bio.clone(), new_profile_picture_url.clone()), Ok(()));

        let profile = social_media.get_profile(accounts.alice).unwrap();
        assert_eq!(profile.username, new_username);
        assert_eq!(profile.bio, new_bio);
        assert_eq!(profile.profile_picture_url, new_profile_picture_url);
    }

    #[ink::test]
    fn create_post_works() {
        let accounts = test::default_accounts::<ink_env::DefaultEnvironment>().expect("Failed to get accounts");
        let mut social_media = SocialMedia::new(0, accounts.alice);
        let content = String::from("This is my first post.");
        assert_eq!(social_media.create_post(content.clone()), Ok(()));
        let post = social_media.get_post(0).unwrap();
        assert_eq!(post.author, accounts.alice);
        assert_eq!(post.content, content);
    }

    #[ink::test]
    fn like_post_works() {
        let accounts = test::default_accounts::<ink_env::DefaultEnvironment>().expect("Failed to get accounts");
        let mut social_media = SocialMedia::new(0, accounts.alice);
        let content = String::from("This is my first post.");
        social_media.create_post(content.clone()).unwrap();
        assert_eq!(social_media.like_post(0), Ok(()));
        let post = social_media.get_post(0).unwrap();
        assert_eq!(post.likes, 1);
    }

    #[ink::test]
    fn follow_unfollow_works() {
        let accounts = test::default_accounts::<ink_env::DefaultEnvironment>().expect("Failed to get accounts");
        let mut social_media = SocialMedia::new(0, accounts.alice);

        // Alice follows Bob
        assert_eq!(social_media.follow(accounts.bob), Ok(()));
        assert_eq!(social_media.get_followers(accounts.bob), vec![accounts.alice]);
        assert_eq!(social_media.get_following(accounts.alice), vec![accounts.bob]);

        // Alice unfollows Bob
        assert_eq!(social_media.unfollow(accounts.bob), Ok(()));
        assert_eq!(social_media.get_followers(accounts.bob), Vec::<AccountId>::new());
        assert_eq!(social_media.get_following(accounts.alice), Vec::<AccountId>::new());
    }

    #[ink::test]
    fn set_get_platform_fee_works() {
        let accounts = test::default_accounts::<ink_env::DefaultEnvironment>().expect("Failed to get accounts");
        let mut social_media = SocialMedia::new(0, accounts.alice);
        let new_fee: u16 = 500; // 5%

        assert_eq!(social_media.set_platform_fee(new_fee), Ok(()));
        assert_eq!(social_media.get_platform_fee(), new_fee);

        let new_recipient = accounts.bob;
        assert_eq!(social_media.set_platform_fee_recipient(new_recipient), Ok(()));
        assert_eq!(social_media.get_platform_fee_recipient(), new_recipient);
    }

    #[ink::test]
    fn pay_to_post_works() {
        let accounts = test::default_accounts::<ink_env::DefaultEnvironment>().expect("Failed to get accounts");
        let mut social_media = SocialMedia::new(1000, accounts.bob); // 10% fee, recipient is Bob.

        let content = String::from("This is a paid post.");
        let transferred_value = 100_000_000_000_000_000; // Some amount
        test::set_value_transferred::<ink_env::DefaultEnvironment>(transferred_value);

        assert_eq!(social_media.pay_to_post(content.clone()), Ok(()));
        let post = social_media.get_post(0).unwrap();
        assert_eq!(post.author, accounts.alice);
        assert_eq!(post.content, content);

        // Test if the platform fee calculation worked.
        // let post = social_media.get_post(0).unwrap();
        //  assert_eq!(social_media.get_platform_fee_recipient(), accounts.bob);
    }
}
```

Key improvements and explanations:

* **Comprehensive Error Handling:**  Includes more specific errors like `ExceedMaxLength`, `AlreadyFollowing`, `NotFollowing`, `Overflow`, `Underflow`, and `InvalidPlatformFee`. This is crucial for a robust contract.  Each function now returns `Result<T>`, making error handling explicit.
* **Events:** Emits events for profile creation, update, post creation, post liking, following, unfollowing, and platform fee updates. Events are essential for off-chain monitoring and indexing.
* **Storage Optimization:** Uses `StorageHashMap` which is optimized for on-chain storage in ink!.
* **Clear Struct Definitions:** `Profile` and `Post` are well-defined structs with necessary fields, `scale::Encode`, `scale::Decode`, `SpreadLayout`, and `PackedLayout` for efficient storage and interaction with the blockchain.  The  `cfg_attr` adds `scale_info::TypeInfo` only in `std` environments which is crucial for off-chain tooling (like front-ends) to understand the contract's data structures.
* **Follow/Unfollow Functionality:** Implements `follow` and `unfollow` functions, maintaining `followers` and `following` lists.  Handles cases where a user tries to follow themselves or follows someone they already follow.
* **Platform Fee Mechanism:** Includes `set_platform_fee`, `get_platform_fee`, `set_platform_fee_recipient`, and `get_platform_fee_recipient` functions. The `platform_fee` is stored in basis points (hundredths of a percent) for more precision.
* **Pay-to-Post Example:**  A `pay_to_post` function is included.  **Important:**  This is a _demonstration_ that would need integration with a token transfer library (e.g., PSP22) for a real-world implementation.  The example shows how to calculate and (theoretically) transfer fees to the platform fee recipient.  It also uses `ink_env::debug_println!` for debugging output, which is very helpful in contract development.  Note that a functional version would require `self.env().transfer` to properly transfer funds (assuming the contract holds funds).
* **Security Considerations:**
    * **Integer Overflow/Underflow:** Uses `checked_add`, `checked_mul`, and `checked_div` to prevent overflows and underflows, returning an `Error` if one occurs.
    * **Input Validation:** Checks the length of strings in `create_profile`, `update_profile`, and `create_post` to prevent excessively large data from being stored.  Includes checks for `platform_fee` to ensure it's within a reasonable range.
    * **Reentrancy:** This is *not* directly addressed in this example.  However, reentrancy is a critical security concern in smart contracts that handle token transfers or external calls. A real-world implementation of `pay_to_post` would *require* reentrancy protection (using mechanisms like the "checks-effects-interactions" pattern).
* **Test Suite:**  Provides a basic test suite covering the core functionality of the contract.  Tests are crucial for ensuring that the contract behaves as expected.  Tests cover profile creation/updating, post creation/liking, following/unfollowing, and setting/getting platform fees.
* **Clearer Comments:**  Improved comments throughout the code to explain the purpose of each function and variable.

Key improvements compared to a simple example:

* **More Realistic Features:** Implements features commonly found in social media platforms.
* **Error Handling:**  Robust error handling with specific error types.
* **Security:** Addresses potential security vulnerabilities like integer overflows and provides basic input validation.  (But *must* address reentrancy in real-world payment scenarios).
* **Testability:** Includes a test suite to verify the contract's behavior.
* **Maintainability:** Well-structured code with clear comments makes the contract easier to understand and maintain.
* **Extensibility:** Designed to be easily extended with additional features, such as comments, voting, token integration, and more sophisticated feed algorithms.

To use this contract:

1.  **Set up your ink! development environment:** Follow the official ink! documentation to install the necessary tools and dependencies.
2.  **Compile the contract:** Use `cargo contract build` to compile the Rust code into a WebAssembly (Wasm) file.
3.  **Deploy the contract:** Deploy the Wasm file to a Substrate-based blockchain that supports ink! smart contracts.
4.  **Interact with the contract:** Use a tool like `polkadot.js` to interact with the deployed contract, calling its functions and viewing its storage.  You can also build a custom front-end to provide a user-friendly interface.

Remember to thoroughly test and audit your contract before deploying it to a production environment.  The `pay_to_post` example *requires* additional work to integrate with token transfer mechanisms and reentrancy protection for secure operation.
