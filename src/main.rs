mod rpc_client;
mod wallet;
use std::vec;
use bdk_bitcoind_rpc::bitcoincore_rpc::RpcApi;
use bdk_wallet::bitcoin::{absolute::Height, transaction::Version, Amount, OutPoint, ScriptBuf, Sequence, Transaction, TxIn, TxOut, Witness, consensus};
use rpc_client::BitcoinRpcClient;
use wallet::wallet::BitcoinWallet;
use bdk_wallet::bitcoin::Network;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    // Initialize RPC client (Using some hard coded variables for the sake of this demo)
    let rpc_client = BitcoinRpcClient::new()?;
    let rpc_url = "http://192.168.3.89:18443".to_string();
    let auth = bdk_bitcoind_rpc::bitcoincore_rpc::Auth::UserPass(
        "reese".to_string(),
        "reesebtc".to_string(),
    );

    // Initialize wallet (Using the BDK defaults becuase our coins cant be rugged on regtest)
    let wallet_path = PathBuf::from("./wallet_data/regtest_wallet.db");
    let descriptor = "wpkh(tprv8ZgxMBicQKsPdcAqYBpzAFwU5yxBUo88ggoBqu1qPcHUfSbKK1sKMLmC7EAk438btHQrSdu3jGGQa6PA71nvH5nkDexhLteJqkM4dQmWF9g/84'/1'/0'/0/*)";
    let change_descriptor = "wpkh(tprv8ZgxMBicQKsPdcAqYBpzAFwU5yxBUo88ggoBqu1qPcHUfSbKK1sKMLmC7EAk438btHQrSdu3jGGQa6PA71nvH5nkDexhLteJqkM4dQmWF9g/84'/1'/0'/1/*)";    
    let mut wallet = BitcoinWallet::new(wallet_path, descriptor, change_descriptor, Network::Regtest)?;
    
    // Get a new address from the wallet and mine some bitcoin to it if we dont have any?
    let receive_address = wallet.wallet.reveal_next_address(bdk_wallet::KeychainKind::External).address;
    println!("Generated wallet address\r\nDeposit Funds and return application to create your TRUC transaction package\r\naddress: {}", receive_address);
    let blocks_to_mine = { if wallet.get_balance().eq(&0) { 100 } else { 1 } };
    let _ = rpc_client.get_client().generate_to_address(blocks_to_mine, &receive_address);

    println!("Syncing wallet...");
    let p2a_script_pubkey: ScriptBuf = bdk_wallet::bitcoin::ScriptBuf::from_bytes(vec![0x51, 0x02, 0x4e, 0x73]);
    let mut signing_options = bdk_wallet::SignOptions::default();
    signing_options.allow_all_sighashes = true;

    wallet.sync_with_node(rpc_url.clone(), auth.clone(), 0)?;

    // Create the p2a (pay to anchor) TRUC parent
    let mut tx_builder = wallet.wallet.build_tx();
    tx_builder.add_recipient(p2a_script_pubkey.clone(), Amount::from_sat(777));
    tx_builder.fee_absolute(Amount::from_sat(0));
    tx_builder.ordering(bdk_wallet::TxOrdering::Untouched);
    tx_builder.version(3);
    let mut truc_parent = tx_builder.finish()
        .map_err(|e| format!("Failed to create TRUC parent PSBT: {}", e))?;
    wallet.wallet.sign(&mut truc_parent, signing_options.clone()).unwrap();
    let parent_tx = truc_parent.extract_tx()?;

    // Create the p2a TRUC child transaction Template
    let mut child_tx = Transaction {
        version: Version::non_standard(3),
        lock_time: bdk_wallet::bitcoin::absolute::LockTime::Blocks(Height::ZERO),
        input: vec![],
        output: vec![],
    };

    // Attach our Anchor input
    let txin = TxIn {
        previous_output: OutPoint {
            txid: parent_tx.compute_txid(),
            vout: 0,
        },
        script_sig: ScriptBuf::default(),
        sequence: Sequence(0),
        witness: Witness::default(),
    };
    child_tx.input.push(txin);
    
    // Attach an Output to the transaction
    let txout = TxOut {
        value: Amount::from_sat(0),
        script_pubkey: {
            let data = "TRUC'R 🚚".as_bytes();
            let mut script = vec![0x6a]; // OP_RETURN
            script.push(data.len() as u8); // Push data length
            script.extend_from_slice(data); // Push data
            ScriptBuf::from_bytes(script)
        },
    };
    child_tx.output.push(txout);

    // Print transaction hexes
    println!("\nParent Transaction Hex:");
    println!("{}", consensus::encode::serialize_hex(&parent_tx));
    println!("\nChild Transaction Hex:");
    println!("{}", consensus::encode::serialize_hex(&child_tx));

    // Submit both transactions as a package
    let package = vec![parent_tx, child_tx];
    let result = rpc_client.submit_package(&package)?;
    println!("Package submission result:\n{}", serde_json::to_string_pretty(&result)?);

    // Print final wallet balance
    let balance = wallet.get_balance();
    println!("Wallet balance: {} sats", balance);

    Ok(())
}
