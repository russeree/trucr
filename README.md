# 🚚 TRUCR

TRUCR is a Rust-based Bitcoin transaction package tool that demonstrates the power of Topologically Restricted Until Confirmed (TRUC) transactions.

## 🌟 Features

- 🏦 Built-in Bitcoin wallet management using BDK (Bitcoin Development Kit)
- 🔗 Creates special P2A (Pay-to-Anchor) parent transactions
- 👶 Generates child transactions with custom OP_RETURN data
- 📦 Submits transaction packages to Bitcoin nodes
- 🔄 Automatic wallet syncing with Bitcoin Core
- 💾 Persistent wallet storage with SQLite

## 🚀 Getting Started

### Prerequisites

- Rust toolchain (2024 edition)
- Bitcoin Core node running in regtest mode
- Basic understanding of Bitcoin transactions

### Configuration

Update the RPC credentials in `src/rpc_client/client.rs`:

```rust
let rpc_url = "http://your-node-ip:18443";
let auth = Auth::UserPass(
    "your-username".to_string(),
    "your-password".to_string(),
);
```

### Building

```bash
cargo build --release
```

### Running

```bash
cargo run
```

## 🎯 How It Works

1. 🏗️ Creates a parent transaction with a P2A output
2. 👶 Generates a child transaction that spends from the parent
3. 📝 Adds custom OP_RETURN data to the child transaction
4. 📦 Submits both transactions as a package to the Bitcoin network

## 🔧 Technical Details

- Uses BDK for wallet operations and transaction building
- Implements custom RPC client for Bitcoin Core interaction
- Supports transaction package submission
- Maintains wallet state in SQLite database
- Generates transaction hexes for debugging

## 📁 Project Structure

```
trucr/
├── src/
│   ├── main.rs           # Main application logic
│   ├── rpc_client/       # Bitcoin RPC client implementation
│   └── wallet/           # Wallet management module
├── wallet_data/          # Wallet storage (gitignored)
└── Cargo.toml           # Project dependencies
```

## 🛠️ Development

The project uses several key Rust crates:
- `bdk_wallet`: Bitcoin Development Kit for wallet operations
- `bdk_bitcoind_rpc`: Bitcoin Core RPC interface
- `bdk_sqlite`: SQLite storage for wallet data

## 📜 License

This project is open source and available under the MIT License.
