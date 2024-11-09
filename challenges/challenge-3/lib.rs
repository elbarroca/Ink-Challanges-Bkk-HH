#![cfg_attr(not(feature = "std"), no_std, no_main)]

// # ✒️ Challenge 3: Connect your DAO to the Super DAO with registration and voting
//
// - **Difficulty**: Mid
// - **Submission Criteria:** ink! contract must
//     - Import the Super DAO trait>
//     - Store Super DAO contract address.
//     - Register contract as member of Super DAO - using trait-based contract calling.
//     - Vote on proposals in the Super DAO - using trait-based contract calling.
//     - Create proposals to call another contract - using trait-based contract calling.
//     - E2E test for cross-contract call.
// - **Submission Guidelines:**
//     - Verify with R0GUE DevRel, and post on X.
// - **Prize:** Sub0 Merch & ink! sports towel

#[ink::contract]
mod dao {
    use ink::{
        contract_ref,
        prelude::{string::String, vec},
        storage::StorageVec,
    };
    use minidao_common::*;
    use superdao_traits::{Call, ContractCall, SuperDao, Vote};

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
            let instance = Self {
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
            
            if self.has_voter(caller) {
                return Err(DaoError::VoterAlreadyRegistered);
            }
            
            self.voters.push(&caller);
            Ok(())
        }

        #[ink(message)]
        pub fn deregister_voter(&mut self) -> Result<(), DaoError> {
            let caller = self.env().caller();
            
            if let Some(pos) = (0..self.voters.len())
                .find(|&i| self.voters.get(i).unwrap() == caller) {
                let len = self.voters.len();
                if pos < len - 1 {
                    let last = self.voters.get(len - 1).unwrap();
                    self.voters.set(pos, &last);
                }
                self.voters.pop();
                Ok(())
            } else {
                Err(DaoError::VoterNotRegistered)
            }
        }

        #[ink(message)]
        pub fn has_voter(&self, voter: AccountId) -> bool {
            (0..self.voters.len())
                .any(|i| self.voters.get(i).unwrap() == voter)
        }

        #[ink(message)]
        pub fn create_contract_call_proposal(&mut self) -> Result<(), DaoError> {
            let caller = self.env().caller();
            
            if !self.has_voter(caller) {
                return Err(DaoError::VoterNotRegistered);
            }
            
            // Create a sample contract call
            // Note: In a real implementation, you would want to parameterize this
            let contract_call = ContractCall {
                callee: caller,  // Example: calling back to the sender
                selector: [0; 4].into(), // Example selector
                input: vec![],  // Example empty input
                transferred_value: 0,
                ref_time_limit: 0,
                allow_reentry: false,
            };
            
            // Create the proposal in the Super DAO
            self.superdao.create_proposal(Call::Contract(contract_call));
            
            Ok(())
        }

        #[ink(message)]
        pub fn vote_proposal(&mut self, proposal_id: u32, approve: bool) -> Result<(), DaoError> {
            let caller = self.env().caller();
            
            if !self.has_voter(caller) {
                return Err(DaoError::VoterNotRegistered);
            }
            
            // Cast the vote in the Super DAO
            self.superdao.vote(proposal_id, if approve { Vote::Yes } else { Vote::No });
            
            Ok(())
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn test_create_superdao_contract_call_proposal() {
            // Create a new DAO instance
            let superdao_account = AccountId::from([0x1; 32]);
            let mut dao = Dao::new(String::from("Test DAO"), superdao_account);
            
            // Register a voter
            assert_eq!(dao.register_voter(), Ok(()));
            
            // Create a proposal
            assert_eq!(dao.create_contract_call_proposal(), Ok(()));
            
            // Try to create a proposal with unregistered voter
            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
            assert_eq!(
                dao.create_contract_call_proposal(),
                Err(DaoError::VoterNotRegistered)
            );
        }

        #[ink::test]
        fn test_vote_superdao_proposal() {
            // Create a new DAO instance
            let superdao_account = AccountId::from([0x1; 32]);
            let mut dao = Dao::new(String::from("Test DAO"), superdao_account);
            
            // Register a voter
            assert_eq!(dao.register_voter(), Ok(()));
            
            // Vote on a proposal
            assert_eq!(dao.vote_proposal(1, true), Ok(()));
            
            // Try to vote with unregistered voter
            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
            assert_eq!(
                dao.vote_proposal(1, true),
                Err(DaoError::VoterNotRegistered)
            );
        }
    }
}
