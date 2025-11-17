#![cfg_attr(not(feature = "std"), no_std, no_main)]

/// Privacy-preserving voting contract using homomorphic encryption
/// Note: This is a simplified demonstration. In production, PHE operations
/// would be performed off-chain with only commitments stored on-chain.
#[ink::contract]
mod boundless_voting {
    use ink::storage::Mapping;
    use ink::prelude::vec::Vec;

    /// Voting poll
    #[ink(storage)]
    pub struct BoundlessVoting {
        /// Poll creator
        creator: AccountId,
        /// Poll question
        question: String,
        /// Poll options
        options: Vec<String>,
        /// Encrypted vote tallies (using PHE ciphertexts)
        /// In reality, these would be Paillier ciphertexts
        /// For simplicity, we use Vec<u8> placeholders
        encrypted_tallies: Mapping<u32, Vec<u8>>,
        /// Voters who have already voted
        has_voted: Mapping<AccountId, bool>,
        /// Poll deadline (timestamp)
        deadline: Timestamp,
        /// Whether results have been finalized
        finalized: bool,
        /// Decrypted results (only available after finalization)
        results: Vec<u64>,
    }

    /// Event emitted when a vote is cast
    #[ink(event)]
    pub struct VoteCast {
        #[ink(topic)]
        voter: AccountId,
        /// We don't reveal which option was voted for
        timestamp: Timestamp,
    }

    /// Event emitted when poll is finalized
    #[ink(event)]
    pub struct PollFinalized {
        results: Vec<u64>,
    }

    /// Voting errors
    #[derive(Debug, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub enum Error {
        AlreadyVoted,
        PollEnded,
        PollNotEnded,
        InvalidOption,
        NotCreator,
        AlreadyFinalized,
        InvalidCiphertext,
    }

    pub type Result<T> = core::result::Result<T, Error>;

    impl BoundlessVoting {
        /// Create a new voting poll
        #[ink(constructor)]
        pub fn new(
            question: String,
            options: Vec<String>,
            duration_secs: u64,
        ) -> Self {
            let caller = Self::env().caller();
            let now = Self::env().block_timestamp();
            let option_count = options.len();

            Self {
                creator: caller,
                question,
                options,
                encrypted_tallies: Mapping::default(),
                has_voted: Mapping::default(),
                deadline: now + duration_secs * 1000, // Convert to milliseconds
                finalized: false,
                results: ink::prelude::vec![0u64; option_count],
            }
        }

        /// Get poll question
        #[ink(message)]
        pub fn question(&self) -> String {
            self.question.clone()
        }

        /// Get poll options
        #[ink(message)]
        pub fn options(&self) -> Vec<String> {
            self.options.clone()
        }

        /// Get poll deadline
        #[ink(message)]
        pub fn deadline(&self) -> Timestamp {
            self.deadline
        }

        /// Check if an account has voted
        #[ink(message)]
        pub fn has_voted(&self, account: AccountId) -> bool {
            self.has_voted.get(&account).unwrap_or(false)
        }

        /// Cast a vote (simplified - in production this would be an encrypted vote)
        ///
        /// In a real PHE-based voting system:
        /// 1. Voter encrypts their choice with the poll's public key
        /// 2. Contract homomorphically adds the encrypted vote to the tally
        /// 3. After voting ends, tally is decrypted by the poll creator
        #[ink(message)]
        pub fn vote(&mut self, encrypted_vote: Vec<u8>, option_index: u32) -> Result<()> {
            let caller = self.env().caller();
            let now = self.env().block_timestamp();

            // Check if already voted
            if self.has_voted(caller) {
                return Err(Error::AlreadyVoted);
            }

            // Check if poll has ended
            if now > self.deadline {
                return Err(Error::PollEnded);
            }

            // Validate option index
            if option_index >= self.options.len() as u32 {
                return Err(Error::InvalidOption);
            }

            // Mark as voted
            self.has_voted.insert(caller, &true);

            // In a real implementation, we would:
            // 1. Verify the encrypted_vote is a valid Paillier ciphertext
            // 2. Homomorphically add it to encrypted_tallies[option_index]
            // For this demo, we just store it
            self.encrypted_tallies.insert(option_index, &encrypted_vote);

            self.env().emit_event(VoteCast {
                voter: caller,
                timestamp: now,
            });

            Ok(())
        }

        /// Finalize poll and decrypt results (simplified)
        ///
        /// In production:
        /// 1. Only callable after deadline
        /// 2. Creator provides decryption of all encrypted tallies
        /// 3. Zero-knowledge proof verifies correct decryption
        #[ink(message)]
        pub fn finalize(&mut self, decrypted_results: Vec<u64>) -> Result<()> {
            let caller = self.env().caller();
            let now = self.env().block_timestamp();

            // Only creator can finalize
            if caller != self.creator {
                return Err(Error::NotCreator);
            }

            // Check if poll has ended
            if now <= self.deadline {
                return Err(Error::PollNotEnded);
            }

            // Check if already finalized
            if self.finalized {
                return Err(Error::AlreadyFinalized);
            }

            // Validate results length
            if decrypted_results.len() != self.options.len() {
                return Err(Error::InvalidCiphertext);
            }

            self.results = decrypted_results.clone();
            self.finalized = true;

            self.env().emit_event(PollFinalized {
                results: decrypted_results,
            });

            Ok(())
        }

        /// Get results (only available after finalization)
        #[ink(message)]
        pub fn results(&self) -> Vec<u64> {
            if self.finalized {
                self.results.clone()
            } else {
                ink::prelude::vec![]
            }
        }

        /// Check if poll is finalized
        #[ink(message)]
        pub fn is_finalized(&self) -> bool {
            self.finalized
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn new_works() {
            let voting = BoundlessVoting::new(
                "What is your favorite color?".to_string(),
                ink::prelude::vec![
                    "Red".to_string(),
                    "Blue".to_string(),
                    "Green".to_string(),
                ],
                3600, // 1 hour
            );

            assert_eq!(voting.question(), "What is your favorite color?");
            assert_eq!(voting.options().len(), 3);
            assert!(!voting.is_finalized());
        }

        #[ink::test]
        fn vote_works() {
            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
            let mut voting = BoundlessVoting::new(
                "Test poll".to_string(),
                ink::prelude::vec!["Option A".to_string(), "Option B".to_string()],
                3600,
            );

            // Alice votes
            let encrypted_vote = ink::prelude::vec![1, 2, 3, 4];
            assert!(voting.vote(encrypted_vote, 0).is_ok());
            assert!(voting.has_voted(accounts.alice));

            // Alice tries to vote again
            let encrypted_vote2 = ink::prelude::vec![5, 6, 7, 8];
            assert_eq!(voting.vote(encrypted_vote2, 1), Err(Error::AlreadyVoted));
        }

        #[ink::test]
        fn finalize_works() {
            let mut voting = BoundlessVoting::new(
                "Test poll".to_string(),
                ink::prelude::vec!["A".to_string(), "B".to_string()],
                0, // Already expired
            );

            // Simulate some votes
            let results = ink::prelude::vec![10, 25];
            assert!(voting.finalize(results.clone()).is_ok());
            assert!(voting.is_finalized());
            assert_eq!(voting.results(), results);
        }
    }
}
