#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::env::{DefaultEnvironment, Environment};

pub type AccountId = <DefaultEnvironment as Environment>::AccountId;
pub type Balance = <DefaultEnvironment as Environment>::Balance;
pub type BlockNumber = <DefaultEnvironment as Environment>::BlockNumber;

#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum DaoError {
    // Voter is already registered in the Dao.
    VoterAlreadyRegistered,
    // Voter is not registered yet..
    VoterNotRegistered,
    // Voter already voted the proposal.
    VoterAlreadyVoted,
    // Proposal does not exist in the Dao.
    ProposalDoesNotExist,
    // Prevote period is not ended.
    PrevotePeriodIsNotEnded,
    // No contract address.
    NoContractAddress,
    // Voter has already voted the proposal.
    AlreadyVoted,
}
