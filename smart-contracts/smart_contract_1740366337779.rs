```rust
#![no_std]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use ink::prelude::collections::BTreeMap;
use ink::prelude::vec;
use ink_lang as ink;

#[ink::contract]
mod decentralized_data_marketplace {

    use ink_prelude::string::ToString;

    /// Defines the storage of our contract.
    #[ink(storage)]
    pub struct DecentralizedDataMarketplace {
        /// Owner of the contract.  Can add/remove data providers.
        owner: AccountId,
        /// Mapping from data provider account to provider information.
        data_providers: BTreeMap<AccountId, Provider>,
        /// Mapping from data ID to data information.
        data_listings: BTreeMap<Hash, DataListing>,
        /// Mapping from buyer to data ID they purchased and the price they paid.
        purchases: BTreeMap<(AccountId, Hash), Balance>,
        /// Fee percentage taken on each transaction.  Expressed as a percentage, so 100 = 1%.
        fee_percentage: u128,
        /// Wallet that collects the fees.
        fee_wallet: AccountId,
    }

    /// Represents a data provider.
    #[derive(scale::Encode, scale::Decode, Debug, Clone, PartialEq, Eq)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo)
    )]
    pub struct Provider {
        name: String,
        description: String,
        category: String, // e.g., "Financial", "Weather", "Social Media"
        join_timestamp: Timestamp,
    }

    /// Represents a data listing available for purchase.
    #[derive(scale::Encode, scale::Decode, Debug, Clone, PartialEq, Eq)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo)
    )]
    pub struct DataListing {
        provider: AccountId,
        name: String,
        description: String,
        price: Balance,
        data_hash: Hash, //  Hash of the actual data.  Data is assumed to be stored off-chain.
        category: String, // e.g., "Image", "Text", "Sensor Data"
        listing_timestamp: Timestamp,
        metadata_url: String, // URL to additional metadata, e.g., schema, licensing.
    }


    /// Events for the contract
    #[ink(event)]
    pub enum Event {
        ProviderAdded { account: AccountId, name: String },
        ProviderRemoved { account: AccountId },
        DataListed { data_hash: Hash, name: String, price: Balance, provider: AccountId },
        DataPurchased { buyer: AccountId, data_hash: Hash, price: Balance, provider: AccountId },
        FeePercentageUpdated { old_percentage: u128, new_percentage: u128 },
        FeeWalletUpdated { old_wallet: AccountId, new_wallet: AccountId },
    }

    /// Errors for the contract.
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        NotOwner,
        ProviderAlreadyExists,
        ProviderNotFound,
        DataAlreadyListed,
        DataNotFound,
        InsufficientFunds,
        ZeroPrice,
        PurchaseAlreadyMade,
        InvalidFeePercentage,
    }

    impl DecentralizedDataMarketplace {
        /// Constructor that initializes the contract.
        #[ink(constructor)]
        pub fn new(fee_percentage: u128, fee_wallet: AccountId) -> Self {
            assert!(fee_percentage <= 10_000, "Fee percentage cannot exceed 100%");
            Self {
                owner: Self::env().caller(),
                data_providers: BTreeMap::new(),
                data_listings: BTreeMap::new(),
                purchases: BTreeMap::new(),
                fee_percentage,
                fee_wallet,
            }
        }

        /// Adds a data provider. Only callable by the owner.
        #[ink(message)]
        pub fn add_provider(
            &mut self,
            account: AccountId,
            name: String,
            description: String,
            category: String,
        ) -> Result<(), Error> {
            self.ensure_owner()?;
            if self.data_providers.contains_key(&account) {
                return Err(Error::ProviderAlreadyExists);
            }

            let provider = Provider {
                name: name.clone(),
                description,
                category,
                join_timestamp: Self::env().block_timestamp(),
            };

            self.data_providers.insert(account, provider);
            self.env().emit_event(Event::ProviderAdded { account, name });
            Ok(())
        }

        /// Removes a data provider. Only callable by the owner.
        #[ink(message)]
        pub fn remove_provider(&mut self, account: AccountId) -> Result<(), Error> {
            self.ensure_owner()?;
            if self.data_providers.remove(&account).is_none() {
                return Err(Error::ProviderNotFound);
            }
            self.env().emit_event(Event::ProviderRemoved { account });
            Ok(())
        }

        /// Lists data for sale.
        #[ink(message)]
        pub fn list_data(
            &mut self,
            data_hash: Hash,
            name: String,
            description: String,
            price: Balance,
            category: String,
            metadata_url: String,
        ) -> Result<(), Error> {
            if price == 0 {
                return Err(Error::ZeroPrice);
            }

            let provider = self.env().caller();
            if !self.data_providers.contains_key(&provider) {
                return Err(Error::ProviderNotFound); // Only providers can list data
            }

            if self.data_listings.contains_key(&data_hash) {
                return Err(Error::DataAlreadyListed);
            }

            let data_listing = DataListing {
                provider,
                name: name.clone(),
                description,
                price,
                data_hash,
                category,
                listing_timestamp: Self::env().block_timestamp(),
                metadata_url,
            };

            self.data_listings.insert(data_hash, data_listing);
            self.env().emit_event(Event::DataListed {
                data_hash,
                name,
                price,
                provider,
            });
            Ok(())
        }

        /// Purchases data.
        #[ink(message, payable)]
        pub fn purchase_data(&mut self, data_hash: Hash) -> Result<(), Error> {
            let buyer = self.env().caller();
            let data_listing = self.data_listings.get(&data_hash).ok_or(Error::DataNotFound)?;
            let price = data_listing.price;

            if self.purchases.contains_key(&(buyer, data_hash)) {
                return Err(Error::PurchaseAlreadyMade);
            }

            if self.env().transferred_value() < price {
                return Err(Error::InsufficientFunds);
            }

            // Calculate fee
            let fee = price * self.fee_percentage / 10_000;
            let provider_payout = price - fee;

            // Transfer funds to the provider
            if self.env().transfer(data_listing.provider, provider_payout).is_err() {
                panic!("Transfer to provider failed."); // Handle this more gracefully in production.
            }

            // Transfer fee to the fee wallet
            if self.env().transfer(self.fee_wallet, fee).is_err() {
                panic!("Transfer of fees failed."); // Handle this more gracefully in production.
            }

            // Record the purchase
            self.purchases.insert((buyer, data_hash), price);

            self.env().emit_event(Event::DataPurchased {
                buyer,
                data_hash,
                price,
                provider: data_listing.provider,
            });

            Ok(())
        }

        /// Updates the fee percentage. Only callable by the owner.
        #[ink(message)]
        pub fn update_fee_percentage(&mut self, new_percentage: u128) -> Result<(), Error> {
            self.ensure_owner()?;
            if new_percentage > 10_000 {
                return Err(Error::InvalidFeePercentage);
            }
            let old_percentage = self.fee_percentage;
            self.fee_percentage = new_percentage;
            self.env().emit_event(Event::FeePercentageUpdated { old_percentage, new_percentage });
            Ok(())
        }

        /// Updates the fee wallet. Only callable by the owner.
        #[ink(message)]
        pub fn update_fee_wallet(&mut self, new_wallet: AccountId) -> Result<(), Error> {
            self.ensure_owner()?;
            let old_wallet = self.fee_wallet;
            self.fee_wallet = new_wallet;
            self.env().emit_event(Event::FeeWalletUpdated { old_wallet, new_wallet });
            Ok(())
        }

        /// Gets data listing by hash.
        #[ink(message)]
        pub fn get_data_listing(&self, data_hash: Hash) -> Option<DataListing> {
            self.data_listings.get(&data_hash).cloned()
        }

        /// Checks if a buyer has purchased data.
        #[ink(message)]
        pub fn has_purchased(&self, buyer: AccountId, data_hash: Hash) -> bool {
            self.purchases.contains_key(&(buyer, data_hash))
        }

        /// Gets provider info by account.
        #[ink(message)]
        pub fn get_provider(&self, account: AccountId) -> Option<Provider> {
            self.data_providers.get(&account).cloned()
        }

        /// Gets all data listings of one provider
        #[ink(message)]
        pub fn get_data_listings_by_provider(&self, provider: AccountId) -> Vec<DataListing> {
            self.data_listings
                .values()
                .filter(|listing| listing.provider == provider)
                .cloned()
                .collect()
        }

        /// Gets the contract's fee percentage.
        #[ink(message)]
        pub fn get_fee_percentage(&self) -> u128 {
            self.fee_percentage
        }

        /// Gets the contract's fee wallet.
        #[ink(message)]
        pub fn get_fee_wallet(&self) -> AccountId {
            self.fee_wallet
        }


        /// Helper function to ensure the caller is the owner.
        fn ensure_owner(&self) -> Result<(), Error> {
            if self.owner != self.env().caller() {
                return Err(Error::NotOwner);
            }
            Ok(())
        }
    }


    #[cfg(test)]
    mod tests {
        use super::*;
        use ink_lang as ink;
        use ink_env::{test::DefaultAccounts, AccountId};

        #[ink::test]
        fn it_works() {
            let accounts = DefaultAccounts::new();
            let mut marketplace = DecentralizedDataMarketplace::new(100, accounts.alice); // 1% fee
            let provider_account = AccountId::from([0x01; 32]);
            let data_hash = Hash::from([0x02; 32]);

            // Add a provider
            assert_eq!(
                marketplace.add_provider(
                    provider_account,
                    "Test Provider".to_string(),
                    "A test provider".to_string(),
                    "General".to_string(),
                ),
                Ok(())
            );

            // List data
            assert_eq!(
                marketplace.list_data(
                    data_hash,
                    "Test Data".to_string(),
                    "Test data description".to_string(),
                    100,
                    "Text".to_string(),
                    "http://example.com/metadata".to_string()
                ),
                Ok(())
            );

            // Purchase data
            ink_env::test::set_value_transferred::<ink_env::DefaultEnvironment>(100);
            assert_eq!(marketplace.purchase_data(data_hash), Ok(()));

            // Check if purchased
            assert_eq!(marketplace.has_purchased(accounts.alice, data_hash), true);
        }
    }
}
```

**Summary of the Smart Contract:**

This Rust-based Ink! smart contract implements a decentralized data marketplace on the Substrate blockchain.  The core functionality allows data providers to register, list their datasets (with associated metadata and pricing), and buyers to purchase these datasets.  Crucially, the actual data *itself* is stored off-chain; the contract manages the metadata, ownership, and payment aspects.

**Key Features:**

*   **Data Provider Registration:**  Allows vetted providers to onboard with detailed profiles.  Only the contract owner can add or remove providers.
*   **Data Listing:**  Providers can list datasets with crucial metadata such as name, description, price, a hash for data integrity (assuming data is stored off-chain), category, and a URL pointing to richer metadata.
*   **Data Purchase:** Buyers pay for the data using the chain's native currency. The payment is automatically split between the provider and a fee, determined by a configurable percentage, that is sent to a designated fee wallet.
*   **Access Control:** Only the contract owner can manage providers and update the fee structure. Data listing is restricted to registered providers.  Buyers can only access data they have purchased (tracked on-chain, but actual data access needs to be managed off-chain, likely using the `data_hash` as a key).
*   **Fees:** The contract implements a transaction fee mechanism, allowing the platform operator to monetize the marketplace.  The fee percentage and fee wallet are configurable by the owner.
*   **Events:** The contract emits events for important actions (provider added/removed, data listed/purchased, fee structure changes), allowing off-chain services to track marketplace activity.
*   **Error Handling:** Provides detailed error types to ensure robust operation and easier debugging.
*   **Off-Chain Data Storage Assumption:** The contract *does not* store the actual data. It assumes that the data itself is hosted off-chain (e.g., IPFS, centralized storage), and that the `data_hash` is a cryptographic commitment to the data's integrity.  The `metadata_url` provides a pointer to where more information about the dataset (schema, licensing terms) can be found.
*   **Queries:** Provides query functions for listing data, checking purchase status, and retrieving provider information.

**Trendy Aspects:**

*   **Decentralized Data Ownership:** The marketplace promotes data ownership by individuals and organizations, bypassing centralized data monopolies.
*   **Data Monetization:**  Allows data providers to directly monetize their data assets in a transparent and secure manner.
*   **Data Provenance:**  The blockchain provides a transparent and immutable record of data transactions, enabling data provenance tracking.
*   **Composable Data Economy:**  This contract can be a building block for a larger data economy, enabling data-driven applications and services.

**Example Use Cases:**

*   A marketplace for environmental sensor data.
*   A platform for researchers to buy and sell anonymized datasets.
*   A decentralized alternative to traditional data aggregators.

**Important Considerations:**

*   **Off-Chain Data Management:** This is the most complex aspect.  The contract relies on the trustworthiness of off-chain data storage.  Data integrity and availability depend on the chosen off-chain solution. Consider using technologies like IPFS or Arweave with cryptographic pinning for increased resilience.
*   **Data Licensing:**  The contract *does not* enforce data licenses. The `metadata_url` can point to license agreements, but enforcement requires off-chain mechanisms.
*   **Scalability:** Blockchain transactions have limited throughput.  A high volume of data purchases could lead to congestion.  Layer-2 scaling solutions might be needed.
*   **Data Quality:** The contract doesn't verify the quality or accuracy of the data.  Reputation systems and data validation mechanisms may be necessary.
*   **Privacy:**  Considerations are needed to protect the privacy of individuals whose data is being bought and sold.  Anonymization techniques and data governance policies are crucial.
*   **Governance:** How the marketplace is governed and evolves is important (e.g., who decides which providers are accepted, how disputes are resolved). A DAO (Decentralized Autonomous Organization) could be integrated for community governance.
