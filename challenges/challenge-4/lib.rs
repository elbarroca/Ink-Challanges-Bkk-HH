#![cfg_attr(not(feature = "std"), no_std, no_main)]

// # ✒️ Challenge 4: Support creating cross-chain proposals to the Super DAO

// - **Difficulty**: Advanced
// - **Submission Criteria:** ink! contract must
//     - Support creating cross-chain proposals to the Super DAO (XCM)
//     - A deployed contract on Pop Network Testnet
//     - Have a cross-chain proposal successfully executed
// - **Submission Guidelines:**
//     - Verify with R0GUE DevRel, and post on X.
// - **Prize:** Sub0 merch

#[ink::contract]
mod dao {
    use ink::{contract_ref, prelude::string::String, storage::StorageVec, xcm::prelude::*};
    use minidao_common::*;
    use superdao_traits::{Call, ChainCall, SuperDao, Vote, ProposalStatus};

    /// Represents a proposal in the DAO
    #[derive(scale::Encode, scale::Decode, Debug)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct Proposal {
        chain_id: u32,
        call_data: Vec<u8>,
        status: ProposalStatus,
        votes_yes: u32,
        votes_no: u32,
    }

    #[ink(storage)]
    pub struct Dao {
        superdao: contract_ref!(SuperDao),
        voters: StorageVec<AccountId>,
        name: String,
        proposals: StorageVec<Proposal>,
        next_proposal_id: u32,
    }

    impl Dao {
        #[ink(constructor)]
        pub fn new(name: String, superdao: AccountId) -> Self {
            let mut instance = Self {
                name,
                superdao: superdao.into(),
                voters: StorageVec::new(),
                proposals: StorageVec::new(),
                next_proposal_id: 0,
            };
            
            // Register this DAO with the SuperDAO
            instance.superdao.register_dao();
            
            instance
        }

        #[ink(message)]
        pub fn get_name(&self) -> String {
            self.name.clone()
        }

        #[ink(message)]
        pub fn register_voter(&mut self) -> Result<(), DaoError> {
            let caller = self.env().caller();
            
            if self.has_voter(caller) {
                return Err(DaoError::VoterAlreadyRegistered);
            }

            self.voters.push(caller);
            Ok(())
        }

        #[ink(message)]
        pub fn deregister_voter(&mut self) -> Result<(), DaoError> {
            let caller = self.env().caller();
            
            let position = self.voters.iter()
                .position(|&voter| voter == voter)
                .ok_or(DaoError::VoterNotRegistered)?;
                
            self.voters.swap_remove(position);
            Ok(())
        }

        #[ink(message)]
        pub fn has_voter(&self, voter: AccountId) -> bool {
            self.voters.iter().any(|&v| v == voter)
        }

        #[ink(message)]
        pub fn create_superdao_cross_chain_proposal(
            &mut self,
            target_chain_id: u32,
            call_data: Vec<u8>
        ) -> Result<u32, DaoError> {
            let caller = self.env().caller();
            
            if !self.has_voter(caller) {
                return Err(DaoError::VoterNotRegistered);
            }

            // Create the cross-chain call
            let chain_call = ChainCall {
                chain_id: target_chain_id,
                call: Call::Custom(call_data.clone()),
            };

            // Create proposal in SuperDAO
            let proposal_id = self.next_proposal_id;
            self.superdao.create_proposal(chain_call);

            // Store proposal locally
            let proposal = Proposal {
                chain_id: target_chain_id,
                call_data,
                status: ProposalStatus::Active,
                votes_yes: 0,
                votes_no: 0,
            };
            
            self.proposals.push(proposal);
            self.next_proposal_id += 1;
            
            Ok(proposal_id)
        }

        #[ink(message)]
        pub fn vote_proposal(&mut self, proposal_id: u32, vote: bool) -> Result<(), DaoError> {
            let caller = self.env().caller();
            
            if !self.has_voter(caller) {
                return Err(DaoError::VoterNotRegistered);
            }

            // Ensure proposal exists
            if proposal_id >= self.next_proposal_id {
                return Err(DaoError::ProposalNotFound);
            }

            let mut proposal = &mut self.proposals[proposal_id as usize];
            
            // Update local vote count
            if vote {
                proposal.votes_yes += 1;
            } else {
                proposal.votes_no += 1;
            }

            // Cast vote in SuperDAO
            let vote_type = if vote { Vote::Yes } else { Vote::No };
            self.superdao.vote(proposal_id, vote_type);
            
            Ok(())
        }

        #[ink(message)]
        pub fn get_proposal(&self, proposal_id: u32) -> Result<Proposal, DaoError> {
            if proposal_id >= self.next_proposal_id {
                return Err(DaoError::ProposalNotFound);
            }
            
            Ok(self.proposals[proposal_id as usize].clone())
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn test_complete_proposal_flow() {
            // Setup
            let name = String::from("Test DAO");
            let superdao_account = AccountId::from([0x1; 32]);
            let mut dao = Dao::new(name, superdao_account);
            
            // Register voter
            let result = dao.register_voter();
            assert!(result.is_ok());
            
            // Create cross-chain proposal
            let call_data = vec![1, 2, 3, 4]; // Example call data
            let proposal_id = dao.create_superdao_cross_chain_proposal(1, call_data.clone())
                .expect("Failed to create proposal");
            
            // Vote on proposal
            let result = dao.vote_proposal(proposal_id, true);
            assert!(result.is_ok());
            
            // Verify proposal state
            let proposal = dao.get_proposal(proposal_id).expect("Failed to get proposal");
            assert_eq!(proposal.votes_yes, 1);
            assert_eq!(proposal.votes_no, 0);
            assert_eq!(proposal.chain_id, 1);
            assert_eq!(proposal.call_data, call_data);
        }

        #[ink::test]
        fn test_invalid_voter() {
            let name = String::from("Test DAO");
            let superdao_account = AccountId::from([0x1; 32]);
            let mut dao = Dao::new(name, superdao_account);
            
            // Try to vote without registering
            let result = dao.vote_proposal(0, true);
            assert!(matches!(result, Err(DaoError::VoterNotRegistered)));
        }
    }
}
