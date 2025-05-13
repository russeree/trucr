mod rpc_client;
mod wallet;
use std::vec;
use bdk_wallet::bitcoin::{absolute::Height, transaction::Version, Amount, OutPoint, ScriptBuf, Sequence, Transaction, TxIn, TxOut, Witness, consensus};
use rpc_client::BitcoinRpcClient;
use wallet::wallet::BitcoinWallet;
use bdk_wallet::bitcoin::Network;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {

    // Initialize RPC client (Using some hard coded variables for the sake of this demo)
    let rpc_client = BitcoinRpcClient::new()?;
    let rpc_url = "http://192.168.1.2:8332".to_string();
    let auth = bdk_bitcoind_rpc::bitcoincore_rpc::Auth::UserPass(
        "reese".to_string(),
        "reesebtc".to_string(),
    );

    // Initialize wallet (Using the BDK defaults becuase our coins cant be rugged on regtest)
    let wallet_path = PathBuf::from("./wallet_data/regtest_wallet.db");
    let descriptor = "wpkh(xprv9s21ZrQH143K2oRrZ5T4HR2fSMhUHkJUTzWL6kWsGGSqE6yQYRvKLTp7oQr4KygEpULjvhb1ezD2Ypth54HckUGy81KXydGJXs1eDm56fda/84'/1'/0'/0/*)";
    let change_descriptor = "wpkh(xprv9s21ZrQH143K2oRrZ5T4HR2fSMhUHkJUTzWL6kWsGGSqE6yQYRvKLTp7oQr4KygEpULjvhb1ezD2Ypth54HckUGy81KXydGJXs1eDm56fda/84'/1'/0'/1/*)";
    let mut wallet = BitcoinWallet::new(wallet_path, descriptor, change_descriptor, Network::Bitcoin)?;

    // Get a new address from the wallet and mine some bitcoin to it if we dont have any?
    let receive_address = wallet.wallet.reveal_next_address(bdk_wallet::KeychainKind::External).address;
    println!("Generated wallet address\r\nDeposit Funds and return application to create your TRUC transaction package\r\naddress: {}", receive_address);
    println!("Syncing wallet...");

    wallet.sync_with_node(rpc_url.clone(), auth.clone(), 893600)?;

    let p2a_script_pubkey: ScriptBuf = bdk_wallet::bitcoin::ScriptBuf::from_bytes(vec![0x51, 0x02, 0x4e, 0x73]);
    let mut signing_options = bdk_wallet::SignOptions::default();
    signing_options.allow_all_sighashes = true;
    signing_options.trust_witness_utxo;

    // Attach an Output to the transaction(s)
    let txout_parent = TxOut {
        value: Amount::from_sat(0),
        script_pubkey: {
            let data = "https://x.com/HomeMiningPod/status/1921361326049259855, this is completely possible to do and doesn't cost 80 dollars. Just use Slipstream to submit your TX hex, See how easy that was.".as_bytes();
            let mut script = vec![0x6A,0x4C, data.len() as u8 + 1]; // OP_RETURN
            script.push(data.len() as u8); // Push data length
            script.extend_from_slice(data); // Push data
            ScriptBuf::from_bytes(script)
        },
    };

    // Create the p2a (pay to anchor) TRUC parent
    let mut tx_builder = wallet.wallet.build_tx();
    tx_builder.add_recipient(p2a_script_pubkey.clone(), Amount::from_sat(377));
    tx_builder.ordering(bdk_wallet::TxOrdering::Untouched);
    tx_builder.version(2);
    tx_builder.fee_absolute(Amount::from_sat(1200));
    let mut truc_parent = tx_builder.finish()
        .map_err(|e| format!("Failed to create TRUC parent PSBT: {}", e))?;
    truc_parent.unsigned_tx.output.push(txout_parent.clone());
    wallet.wallet.sign(&mut truc_parent, signing_options.clone()).unwrap();
    let parent_tx = truc_parent.extract_tx()?;


    // Print transaction hexes
    println!("\nParent Transaction Hex:");
    println!("{}", consensus::encode::serialize_hex(&parent_tx));

    // Submit both transactions as a package
    //let package = vec![parent_tx, child_tx];
    //let result = rpc_client.submit_package(&package)?;
    //println!("Package submission result:\n{}", serde_json::to_string_pretty(&result)?);

    // Print final wallet balance
    let balance = wallet.get_balance();
    println!("Wallet balance: {} sats", balance);

    Ok(())
}
