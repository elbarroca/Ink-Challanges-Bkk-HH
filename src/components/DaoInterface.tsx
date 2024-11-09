import { useState } from 'react';
import { useDao } from '@/hooks/useDao';
import { ProposalList } from './ProposalList';
import { VoterRegistration } from './VoterRegistration';
import { CreateProposal } from './CreateProposal';

export function DaoInterface() {
  const [address, setAddress] = useState<string>();
  const { isVoter, registerVoter, voteOnProposal } = useDao(address);

  const connectWallet = async () => {
    // Implementation for connecting to Polkadot wallet
    const { web3Enable, web3Accounts } = await import('@polkadot/extension-dapp');
    await web3Enable('DAO UI');
    const accounts = await web3Accounts();
    if (accounts[0]) {
      setAddress(accounts[0].address);
    }
  };

  return (
    <div className="space-y-6">
      <div className="bg-white shadow rounded-lg p-6">
        <h1 className="text-2xl font-bold text-gray-900 mb-4">DAO Dashboard</h1>
        
        {!address ? (
          <button
            onClick={connectWallet}
            className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700"
          >
            Connect Wallet
          </button>
        ) : (
          <div className="space-y-6">
            <VoterRegistration 
              isVoter={isVoter} 
              onRegister={registerVoter.mutate}
              address={address}
            />
            
            {isVoter && (
              <>
                <CreateProposal />
                <ProposalList onVote={voteOnProposal.mutate} />
              </>
            )}
          </div>
        )}
      </div>
    </div>
  );
} 