#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod dao {
    use ink::{
        contract_ref,
        prelude::{string::String, vec::Vec},
        storage::{Mapping, StorageVec},
        xcm::prelude::*,
    };
    use minidao_common::*;
    use pop_api::v0::fungibles::traits::Psp22;
    use superdao_traits::{Call, ChainCall, ContractCall, SuperDao, Vote};

    const VOTING_PERIOD: BlockNumber = 100; // Number of blocks for voting period
    const MINT_AMOUNT: Balance = 100; // Amount of tokens to mint for each voter

    #[derive(Clone, Default)]
    #[cfg_attr(
        feature = "std",
        derive(Debug, PartialEq, Eq, ink::storage::traits::StorageLayout)
    )]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub struct Prevote {
        pub deadline: BlockNumber,
        pub aye_votes: Vec<(AccountId, Balance)>,
        pub nay_votes: Vec<(AccountId, Balance)>,
    }

    #[ink(storage)]
    pub struct Dao {
        name: String,
        prevotes: Mapping<u32, Prevote>,
        voters: StorageVec<AccountId>,
        token: AccountId,
        superdao: contract_ref!(SuperDao),
    }

    impl Dao {
        #[ink(constructor)]
        pub fn new(name: String, superdao: AccountId, token: AccountId) -> Self {
            Self {
                name,
                token,
                superdao: superdao.into(),
                voters: StorageVec::new(),
                prevotes: Mapping::new(),
            }
        }

        #[ink(message)]
        pub fn name(&self) -> String {
            self.name.clone()
        }

        #[ink(message)]
        pub fn register_voter(&mut self) -> Result<(), DaoError> {
            let caller = self.env().caller();
            
            // Check if voter is already registered
            if self.has_voter(caller) {
                return Err(DaoError::VoterAlreadyRegistered);
            }

            // Register voter
            self.voters.push(&caller);

            // Mint tokens for the new voter
            let token_contract: contract_ref!(Psp22) = self.token.into();
            token_contract.mint(caller, MINT_AMOUNT)?;

            Ok(())
        }

        #[ink(message)]
        pub fn deregister_voter(&mut self) -> Result<(), DaoError> {
            let caller = self.env().caller();
            
            // Find voter index
            let mut found = false;
            for i in 0..self.voters.len() {
                if self.voters.get(i).unwrap() == caller {
                    self.voters.swap_remove(i);
                    found = true;
                    break;
                }
            }

            if !found {
                return Err(DaoError::VoterNotRegistered);
            }

            Ok(())
        }

        #[ink(message)]
        pub fn has_voter(&self, voter: AccountId) -> bool {
            for i in 0..self.voters.len() {
                if self.voters.get(i).unwrap() == voter {
                    return true;
                }
            }
            false
        }

        #[ink(message)]
        pub fn create_superdao_cross_chain_proposal(&mut self) -> Result<(), DaoError> {
            let caller = self.env().caller();
            if !self.has_voter(caller) {
                return Err(DaoError::VoterNotRegistered);
            }

            // Create cross-chain proposal
            let call = Call::Chain(ChainCall {
                dest: MultiLocation::new(1, X1(Parachain(1000))),
                message: Vec::new(),
            });
            
            let proposal_id = self.superdao.propose(call)?;
            
            // Initialize prevote
            let prevote = Prevote {
                deadline: self.env().block_number() + VOTING_PERIOD,
                aye_votes: Vec::new(),
                nay_votes: Vec::new(),
            };
            self.prevotes.insert(proposal_id, &prevote);

            Ok(())
        }

        #[ink(message)]
        pub fn create_contract_call_proposal(&mut self) -> Result<(), DaoError> {
            let caller = self.env().caller();
            if !self.has_voter(caller) {
                return Err(DaoError::VoterNotRegistered);
            }

            // Create contract call proposal
            let call = Call::Contract(ContractCall {
                contract: AccountId::from([0x0; 32]),
                selector: [0x0; 4],
                input: Vec::new(),
            });
            
            let proposal_id = self.superdao.propose(call)?;
            
            // Initialize prevote
            let prevote = Prevote {
                deadline: self.env().block_number() + VOTING_PERIOD,
                aye_votes: Vec::new(),
                nay_votes: Vec::new(),
            };
            self.prevotes.insert(proposal_id, &prevote);

            Ok(())
        }

        #[ink(message)]
        pub fn submit_prevote(&mut self, proposal_id: u32, approved: bool) -> Result<(), DaoError> {
            let caller = self.env().caller();
            if !self.has_voter(caller) {
                return Err(DaoError::VoterNotRegistered);
            }

            // Get voter's token balance
            let token_contract: contract_ref!(Psp22) = self.token.into();
            let balance = token_contract.balance_of(caller);

            // Get and update prevote
            let mut prevote = self.prevotes.get(proposal_id).ok_or(DaoError::ProposalDoesNotExist)?;
            
            if approved {
                prevote.aye_votes.push((caller, balance));
            } else {
                prevote.nay_votes.push((caller, balance));
            }

            self.prevotes.insert(proposal_id, &prevote);
            Ok(())
        }

        #[ink(message)]
        pub fn vote_proposal(&mut self, proposal_id: u32) -> Result<(), DaoError> {
            let caller = self.env().caller();
            if !self.has_voter(caller) {
                return Err(DaoError::VoterNotRegistered);
            }

            // Get prevote
            let prevote = self.prevotes.get(proposal_id).ok_or(DaoError::ProposalDoesNotExist)?;
            
            // Check if voting period has ended
            if self.env().block_number() < prevote.deadline {
                return Err(DaoError::VotingPeriodNotEnded);
            }

            // Calculate total votes
            let total_aye: Balance = prevote.aye_votes.iter().map(|(_, balance)| balance).sum();
            let total_nay: Balance = prevote.nay_votes.iter().map(|(_, balance)| balance).sum();

            // Submit final vote if aye votes win
            if total_aye > total_nay {
                self.superdao.vote(proposal_id, Vote::Aye)?;
            }

            Ok(())
        }
    }
}
