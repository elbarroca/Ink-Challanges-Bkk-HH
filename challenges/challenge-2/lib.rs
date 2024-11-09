#![cfg_attr(not(feature = "std"), no_std, no_main)]

// # ✒️ Challenge 2: Implement voter registration, proposal management, and voting in your Dao.
//
// - **Difficulty**: Mid
// - **Submission Criteria:** ink! contract must
//     - Use a storage-optimized data-structure `Mapping` or `StorageVec`
//     - Store registered members, member votes, and proposals to vote on.
//     - Implement methods to register and de-register members.
//     - Implement methods to create proposals and a method to vote on proposals.
//     - Unit tests for adding members, votes, and proposals.
// - **Submission Guidelines:**
//     - Verify with R0GUE DevRel, and post on X.
// - **Prize:** sub0 merch

#[ink::contract]
mod dao {
    use ink::{
        prelude::string::String,
        storage::Mapping,
    };
    use minidao_common::*;

    #[derive(Clone)]
    #[cfg_attr(
        feature = "std",
        derive(Debug, PartialEq, Eq, ink::storage::traits::StorageLayout)
    )]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub struct BasicProposal {
        pub vote_count: u32,
    }

    #[ink(storage)]
    pub struct Dao {
        /// Name of the DAO
        name: String,
        /// Mapping of registered voters
        voters: Mapping<AccountId, bool>,
        /// Mapping of proposals
        proposals: Mapping<u32, BasicProposal>,
        /// Mapping of voter's vote count
        vote_counts: Mapping<AccountId, u32>,
        /// Mapping to track if a voter has voted on a specific proposal
        has_voted: Mapping<(AccountId, u32), bool>,
        /// Counter for proposal IDs
        next_proposal_id: u32,
    }

    impl Dao {
        // Constructor that initializes the values for the contract.
        #[ink(constructor)]
        pub fn new(name: String) -> Self {
            Self {
                name,
                voters: Mapping::default(),
                proposals: Mapping::default(),
                vote_counts: Mapping::default(),
                has_voted: Mapping::default(),
                next_proposal_id: 0,
            }
        }

        // Constructor that initializes the default values for the contract.
        #[ink(constructor)]
        pub fn default() -> Self {
            Self::new(String::from("DefaultDAO"))
        }

        #[ink(message)]
        pub fn get_name(&self) -> String {
            self.name.clone()
        }

        #[ink(message)]
        pub fn register_voter(&mut self) -> Result<(), DaoError> {
            let caller = self.env().caller();
            if self.voters.get(&caller).unwrap_or_default() {
                return Err(DaoError::VoterAlreadyRegistered);
            }
            self.voters.insert(&caller, &true);
            Ok(())
        }

        #[ink(message)]
        pub fn deregister_voter(&mut self) -> Result<(), DaoError> {
            let caller = self.env().caller();
            if !self.voters.get(&caller).unwrap_or_default() {
                return Err(DaoError::VoterNotRegistered);
            }
            self.voters.insert(&caller, &false);
            Ok(())
        }

        #[ink(message)]
        pub fn has_voter(&self, voter: AccountId) -> bool {
            self.voters.get(&voter).unwrap_or_default()
        }

        #[ink(message)]
        pub fn create_proposal(&mut self) -> Result<(), DaoError> {
            let caller = self.env().caller();
            if !self.voters.get(&caller).unwrap_or_default() {
                return Err(DaoError::VoterNotRegistered);
            }

            let proposal = BasicProposal { vote_count: 0 };
            self.proposals.insert(&self.next_proposal_id, &proposal);
            self.next_proposal_id += 1;
            Ok(())
        }

        #[ink(message)]
        pub fn remove_proposal(&mut self, proposal_id: u32) -> Result<(), DaoError> {
            let caller = self.env().caller();
            if !self.voters.get(&caller).unwrap_or_default() {
                return Err(DaoError::VoterNotRegistered);
            }

            if self.proposals.get(&proposal_id).is_none() {
                return Err(DaoError::ProposalDoesNotExist);
            }

            self.proposals.remove(&proposal_id);
            Ok(())
        }

        #[ink(message)]
        pub fn get_proposal(&self, proposal_id: u32) -> Option<BasicProposal> {
            self.proposals.get(&proposal_id)
        }

        #[ink(message)]
        pub fn vote(&mut self, proposal_id: u32) -> Result<(), DaoError> {
            let caller = self.env().caller();
            
            if !self.voters.get(&caller).unwrap_or_default() {
                return Err(DaoError::VoterNotRegistered);
            }

            let mut proposal = self.proposals
                .get(&proposal_id)
                .ok_or(DaoError::ProposalDoesNotExist)?;

            // Check if voter has already voted on this proposal
            if self.has_voted.get(&(caller, proposal_id)).unwrap_or_default() {
                return Err(DaoError::AlreadyVoted);
            }

            // Update proposal vote count
            proposal.vote_count += 1;
            self.proposals.insert(&proposal_id, &proposal);

            // Mark that this voter has voted on this proposal
            self.has_voted.insert(&(caller, proposal_id), &true);

            // Update voter's total vote count
            let current_votes = self.vote_counts.get(&caller).unwrap_or_default();
            self.vote_counts.insert(&caller, &(current_votes + 1));

            Ok(())
        }

        #[ink(message)]
        pub fn vote_count(&self, voter: AccountId) -> u32 {
            self.vote_counts.get(&voter).unwrap_or_default()
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn test_voter_registration() {
            let mut dao = Dao::new(String::from("TestDAO"));
            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();

            // Test registration
            assert!(dao.register_voter().is_ok());
            assert!(dao.has_voter(accounts.alice));

            // Test duplicate registration
            assert!(dao.register_voter().is_err());

            // Test deregistration
            assert!(dao.deregister_voter().is_ok());
            assert!(!dao.has_voter(accounts.alice));

            // Test deregistration of non-registered voter
            assert!(dao.deregister_voter().is_err());
        }

        #[ink::test]
        fn test_proposal_management() {
            let mut dao = Dao::new(String::from("TestDAO"));
            
            // Register voter
            assert!(dao.register_voter().is_ok());

            // Create proposal
            assert!(dao.create_proposal().is_ok());

            // Get proposal
            let proposal = dao.get_proposal(0);
            assert!(proposal.is_some());
            assert_eq!(proposal.unwrap().vote_count, 0);

            // Remove proposal
            assert!(dao.remove_proposal(0).is_ok());
            assert!(dao.get_proposal(0).is_none());
        }

        #[ink::test]
        fn test_vote() {
            let mut dao = Dao::new(String::from("TestDAO"));
            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();

            // Register voter
            assert!(dao.register_voter().is_ok());

            // Create proposal
            assert!(dao.create_proposal().is_ok());

            // Vote on proposal
            assert!(dao.vote(0).is_ok());
            assert_eq!(dao.vote_count(accounts.alice), 1);

            // Check proposal vote count
            let proposal = dao.get_proposal(0).unwrap();
            assert_eq!(proposal.vote_count, 1);

            // Try to vote again on same proposal
            assert!(dao.vote(0).is_err());
        }
    }
}
