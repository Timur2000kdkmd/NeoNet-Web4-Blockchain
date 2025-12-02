import React, { createContext, useContext, useState, useEffect, useCallback } from 'react';
import { NeoWallet, createWallet, restoreWallet, validateMnemonic, injectNeoProvider } from '../lib/wallet';

const WalletContext = createContext(null);

export function useWallet() {
  const context = useContext(WalletContext);
  if (!context) {
    throw new Error('useWallet must be used within a WalletProvider');
  }
  return context;
}

export function WalletProvider({ children }) {
  const [wallet, setWallet] = useState(null);
  const [isConnected, setIsConnected] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState(null);
  const [addresses, setAddresses] = useState({ evmAddress: null, neoAddress: null });
  const [balances, setBalances] = useState({ NEO: 0 });
  const [showMnemonic, setShowMnemonic] = useState(false);
  const [mnemonic, setMnemonic] = useState('');

  useEffect(() => {
    const savedAddresses = localStorage.getItem('neonet_addresses');
    if (savedAddresses) {
      try {
        const data = JSON.parse(savedAddresses);
        setAddresses(data);
      } catch (e) {
        console.error('Failed to parse saved addresses:', e);
      }
    }
  }, []);

  const createNewWallet = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    try {
      const newWallet = await createWallet();
      setWallet(newWallet);
      setAddresses(newWallet.getAddresses());
      setMnemonic(newWallet.getMnemonic());
      setShowMnemonic(true);
      setIsConnected(true);
      
      localStorage.setItem('neonet_addresses', JSON.stringify({
        evmAddress: newWallet.evmAddress,
        neoAddress: newWallet.neoAddress,
        evmPublicKey: newWallet.evmPublicKey,
        neoPublicKey: newWallet.neoPublicKey
      }));
      
      injectNeoProvider(newWallet);
      
      return newWallet;
    } catch (e) {
      setError(e.message);
      throw e;
    } finally {
      setIsLoading(false);
    }
  }, []);

  const importWallet = useCallback(async (mnemonicPhrase) => {
    setIsLoading(true);
    setError(null);
    try {
      if (!validateMnemonic(mnemonicPhrase)) {
        throw new Error('Invalid mnemonic phrase');
      }
      
      const restoredWallet = await restoreWallet(mnemonicPhrase);
      setWallet(restoredWallet);
      setAddresses(restoredWallet.getAddresses());
      setIsConnected(true);
      
      localStorage.setItem('neonet_addresses', JSON.stringify({
        evmAddress: restoredWallet.evmAddress,
        neoAddress: restoredWallet.neoAddress,
        evmPublicKey: restoredWallet.evmPublicKey,
        neoPublicKey: restoredWallet.neoPublicKey
      }));
      
      injectNeoProvider(restoredWallet);
      
      return restoredWallet;
    } catch (e) {
      setError(e.message);
      throw e;
    } finally {
      setIsLoading(false);
    }
  }, []);

  const disconnect = useCallback(() => {
    if (wallet) {
      wallet.disconnect();
    }
    setWallet(null);
    setIsConnected(false);
    setAddresses({ evmAddress: null, neoAddress: null });
    setBalances({ NEO: 0 });
    setMnemonic('');
    localStorage.removeItem('neonet_addresses');
  }, [wallet]);

  const signMessage = useCallback(async (message, mode = 'hybrid') => {
    if (!wallet || !isConnected) {
      throw new Error('Wallet not connected');
    }
    return wallet.signTransaction({ message }, mode);
  }, [wallet, isConnected]);

  const signTransaction = useCallback(async (tx, mode = 'hybrid') => {
    if (!wallet || !isConnected) {
      throw new Error('Wallet not connected');
    }
    return wallet.signTransaction(tx, mode);
  }, [wallet, isConnected]);

  const updateBalances = useCallback((newBalances) => {
    setBalances(prev => ({ ...prev, ...newBalances }));
    if (wallet) {
      wallet.setBalances(newBalances);
    }
  }, [wallet]);

  const value = {
    wallet,
    isConnected,
    isLoading,
    error,
    addresses,
    balances,
    mnemonic,
    showMnemonic,
    setShowMnemonic,
    createNewWallet,
    importWallet,
    disconnect,
    signMessage,
    signTransaction,
    updateBalances,
    evmAddress: addresses.evmAddress,
    quantumAddress: addresses.neoAddress
  };

  return (
    <WalletContext.Provider value={value}>
      {children}
    </WalletContext.Provider>
  );
}

export default WalletProvider;
