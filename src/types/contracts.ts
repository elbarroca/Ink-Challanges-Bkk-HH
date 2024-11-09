export interface DaoContract {
  get_name(): Promise<string>;
  register_voter(): Promise<void>;
  deregister_voter(): Promise<void>;
  has_voter(address: string): Promise<boolean>;
  create_superdao_cross_chain_proposal(): Promise<void>;
  create_contract_call_proposal(): Promise<void>;
  vote_proposal(proposalId: number, vote: boolean): Promise<void>;
}

export interface Proposal {
  id: number;
  description: string;
  votes_for: number;
  votes_against: number;
  status: 'Active' | 'Completed';
} 