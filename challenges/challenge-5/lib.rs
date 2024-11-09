#![cfg_attr(not(feature = "std"), no_std, no_main)]

// # ✒️ Challenge 5: Build a UI for your DAO
//
// NOTE: Using this contract to combine the functionalities of challenge 4 contract and challenge 3 contract. Compile and deploy on Pop Network then building a UI for it.
//
// - **Difficulty:** Mid
// - **Submission Criteria:** The UI must support
//     - Registering & viewing members
//     - Voting on and viewing proposals
//     - Viewing overall proposal votes
// - **Submission Guidelines:** Verify with R0GUE or Dedot DevRel, and post on X
// - **Prize:** Sub0 merch & ink! sports towel

#[ink::contract]
mod dao {
    use ink::{
        contract_ref,
        prelude::{string::String, vec},
        storage::StorageVec,
        xcm::prelude::*,
    };
    use minidao_common::*;
    use superdao_traits::{Call, ChainCall, ContractCall, SuperDao, Vote};

    #[ink(storage)]
    pub struct Dao {
        superdao: contract_ref!(SuperDao),
        voters: StorageVec<AccountId>,
        name: String,
    }

    impl Dao {
        // Constructor that initializes the values for the contract.
        #[ink(constructor)]
        pub fn new(name: String, superdao: AccountId) -> Self {
            // Register your Dao as a member of the Superdao.
            let mut instance = Self {
                name,
                superdao: superdao.into(),
                voters: StorageVec::new(),
            };
            instance
        }

        #[ink(message)]
        pub fn get_name(&self) -> String {
            self.name.clone()
        }

        #[ink(message)]
        pub fn register_voter(&mut self) -> Result<(), DaoError> {
            let caller = self.env().caller();
            
            // Check if voter is already registered
            if self.has_voter(caller) {
                return Err(DaoError::VoterAlreadyRegistered);
            }

            // Register the new voter
            self.voters.push(caller);
            Ok(())
        }

        #[ink(message)]
        pub fn deregister_voter(&mut self) -> Result<(), DaoError> {
            let caller = self.env().caller();
            
            // Find voter index
            let voter_index = self.voters
                .iter()
                .position(|v| *v == caller)
                .ok_or(DaoError::VoterNotRegistered)?;

            // Remove voter
            self.voters.swap_remove(voter_index);
            Ok(())
        }

        #[ink(message)]
        pub fn has_voter(&self, voter: AccountId) -> bool {
            self.voters.iter().any(|v| *v == voter)
        }

        #[ink(message)]
        pub fn create_superdao_cross_chain_proposal(&mut self) -> Result<(), DaoError> {
            let caller = self.env().caller();
            
            // Check if caller is registered voter
            if !self.has_voter(caller) {
                return Err(DaoError::VoterNotRegistered);
            }

            // Create cross-chain proposal
            let dest = MultiLocation::new(1, X1(Parachain(1000)));
            let message = xcm::VersionedXcm::V3(Xcm(vec![
                WithdrawAsset(MultiAssets::new()),
                BuyExecution {
                    fees: MultiAsset { id: Concrete(MultiLocation::here()), fun: Fungible(1000000000) },
                    weight_limit: Limited(Weight::from_parts(1000000000, 1000000000)),
                },
            ]));

            let call = Call::Chain(ChainCall {
                dest,
                message,
            });

            self.superdao.create_proposal(call);
            Ok(())
        }

        #[ink(message)]
        pub fn create_contract_call_proposal(&mut self) -> Result<(), DaoError> {
            let caller = self.env().caller();
            
            // Check if caller is registered voter
            if !self.has_voter(caller) {
                return Err(DaoError::VoterNotRegistered);
            }

            // Create contract call proposal
            let contract_call = ContractCall {
                dest: caller, // Example destination
                selector: [0; 4], // Example selector
                input: vec![], // Example input
                transfer: 0, // Example transfer amount
                gas_limit: 0, // Example gas limit
            };

            let call = Call::Contract(contract_call);
            self.superdao.create_proposal(call);
            Ok(())
        }

        #[ink(message)]
        pub fn vote_proposal(&mut self, proposal_id: u32, vote: bool) -> Result<(), DaoError> {
            let caller = self.env().caller();
            
            // Check if caller is registered voter
            if !self.has_voter(caller) {
                return Err(DaoError::VoterNotRegistered);
            }

            // Cast vote on proposal
            let vote = if vote { Vote::Yes } else { Vote::No };
            self.superdao.vote_proposal(proposal_id, vote);
            Ok(())
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::dao::Dao;

        #[ink::test]
        fn test_vote_superdao_cross_chain_proposal() {
            todo!("Challenge 4");
        }
    }
}
