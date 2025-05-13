use bdk_bitcoind_rpc::{
    bitcoincore_rpc::{Auth, Client}, Emitter
};

use bdk_wallet::{
    bitcoin::Network, file_store::Store, ChangeSet, KeychainKind, PersistedWallet, Wallet
};
use std::path::PathBuf;
use thiserror::Error;

const DB_MAGIC: &str = "bdk-wallet";

#[allow(dead_code)]
pub struct BitcoinWallet {
    pub wallet: PersistedWallet<Store<ChangeSet>>,
    db: Store<ChangeSet>,
    network: Network,
}

#[derive(Error, Debug, Clone)]
pub enum WalletError {
    Database(String),
    Wallet(String),
    Rpc(String),
}

impl std::fmt::Display for WalletError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WalletError::Database(e) => write!(f, "Database error: {}", e),
            WalletError::Wallet(e) => write!(f, "Wallet error: {}", e),
            WalletError::Rpc(e) => write!(f, "RPC error: {}", e),
        }
    }
}

impl BitcoinWallet {
    pub fn new(
        wallet_path: PathBuf,
        descriptor: &str,
        change_descriptor: &str,
        network: Network,
    ) -> Result<Self, WalletError> {
        // Open or create wallet database
        let mut db = Store::<ChangeSet>::open_or_create_new(DB_MAGIC.as_bytes(), wallet_path)
            .map_err(|e| WalletError::Database(e.to_string()))?;

        // Try to load existing wallet or create new one
        let wallet_opt = Wallet::load()
            .descriptor(KeychainKind::External, Some(descriptor.to_string()))
            .descriptor(KeychainKind::Internal, Some(change_descriptor.to_string()))
            .extract_keys()
            .check_network(network)
            .load_wallet(&mut db)
            .map_err(|e| WalletError::Wallet(e.to_string()))?;

        let wallet = match wallet_opt {
            Some(wallet) => wallet,
            None => Wallet::create(descriptor.to_string(), change_descriptor.to_string())
                .network(network)
                .create_wallet(&mut db)
                .map_err(|e| WalletError::Wallet(e.to_string()))?,
        };

        Ok(Self {
            wallet,
            db,
            network,
        })
    }

    #[allow(dead_code)]
    pub fn get_new_address(&mut self) -> Result<String, WalletError> {
        let address = self.wallet.reveal_next_address(KeychainKind::External);
        self.wallet.persist(&mut self.db)
            .map_err(|e| WalletError::Wallet(e.to_string()))?;
        Ok(address.address.to_string())
    }

    pub fn get_balance(&self) -> u64 {
        self.wallet.balance().total().to_sat()
    }

    pub fn sync_with_node(
        &mut self,
        rpc_url: String,
        auth: Auth,
        start_height: u32,
    ) -> Result<(), WalletError> {
        let wallet_tip = self.wallet.latest_checkpoint();
        let client = Client::new(&rpc_url, auth)
            .map_err(|e| WalletError::Rpc(e.to_string()))?;

        let mut emitter = Emitter::new(&client, wallet_tip, start_height);
        
        while let Ok(Some(block_event)) = emitter.next_block() {
            self.wallet.apply_block_connected_to(&block_event.block, block_event.block_height(), block_event.connected_to())
                .map_err(|e| WalletError::Wallet(e.to_string()))?;

            

            self.wallet.persist(&mut self.db)
                .map_err(|e| WalletError::Database(e.to_string()))?;
        }

        // Process mempool
        if let Ok(mempool) = emitter.mempool() {
            self.wallet.apply_unconfirmed_txs(mempool);
            self.wallet.persist(&mut self.db)
                .map_err(|e| WalletError::Database(e.to_string()))?;
        }

        Ok(())
    }

    #[allow(dead_code)]
    pub fn get_transaction_count(&self) -> usize {
        self.wallet.transactions().count()
    }

    #[allow(dead_code)]
    pub fn get_utxo_count(&self) -> usize {
        self.wallet.list_unspent().count()
    }
}