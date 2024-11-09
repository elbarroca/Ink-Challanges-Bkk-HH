import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { web3FromAddress } from '@polkadot/extension-dapp';
import { ApiPromise, WsProvider } from '@polkadot/api';
import { useState, useEffect } from 'react';

export const useDao = (address: string | undefined) => {
  const queryClient = useQueryClient();
  const [api, setApi] = useState<ApiPromise | null>(null);

  useEffect(() => {
    const initApi = async () => {
      const wsProvider = new WsProvider('wss://your-node-url');
      const api = await ApiPromise.create({ provider: wsProvider });
      setApi(api);
    };
    initApi();
  }, []);

  const { data: isVoter } = useQuery({
    queryKey: ['isVoter', address],
    queryFn: async () => {
      if (!address || !api) return false;
      const contract = await api.query.dao.hasVoter(address);
      return contract;
    },
    enabled: !!address && !!api,
  });

  const registerVoter = useMutation({
    mutationFn: async () => {
      if (!address || !api) throw new Error('Not connected');
      const injector = await web3FromAddress(address);
      await api.tx.dao.registerVoter().signAndSend(address, { signer: injector.signer });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['isVoter'] });
    },
  });

  const voteOnProposal = useMutation({
    mutationFn: async ({ proposalId, vote }: { proposalId: number; vote: boolean }) => {
      if (!address || !api) throw new Error('Not connected');
      const injector = await web3FromAddress(address);
      await api.tx.dao.voteProposal(proposalId, vote)
        .signAndSend(address, { signer: injector.signer });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['proposals'] });
    },
  });

  return {
    isVoter,
    registerVoter,
    voteOnProposal,
  };
}; 