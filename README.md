# Raydium Liquidity Management Tool

This project is a tool written in Rust for interacting with the Raydium CP-AMM and CLMM on the Solana Devnet. It provides a client-side implementation for creating a liquidity pool and, for CP-AMM specifically, managing liquidity without relying on the Anchor framework for client-side operations.

## Features

- **Initialize Pool**: Create a new liquidity pool for a given pair of SPL tokens.
- **Deposit Liquidity**: Add liquidity to an existing CP-AMM pool to mint LP tokens.
- **Withdraw Liquidity**: Burn LP tokens to redeem the underlying assets from the CP-AMM pool.
- **Atomic Operations** for CP-AMM:
    - Atomically deposit and withdraw liquidity in a single transaction.
    - Atomically withdraw and deposit liquidity in a single transaction.

---

## Usage

The `main.rs` file acts as the driver for this tool. To execute a specific action, presently, there is a necessity to add the relevant function call within `main()`.

### Examples:

1.  **CP-AMM create pool `src/main.rs`**: [3dx3Y8pGWtzhsVttJdbVCSNReUPn3pLmFQWW7amWhmdnqVgidJs7Cw5f3Kp5jWK7HYjBpYXjYuMMqZWGruhbDMCt](https://explorer.solana.com/tx/3dx3Y8pGWtzhsVttJdbVCSNReUPn3pLmFQWW7amWhmdnqVgidJs7Cw5f3Kp5jWK7HYjBpYXjYuMMqZWGruhbDMCt?cluster=devnet)
```rust
    // src/main.rs

    mod instructions;
    use instructions::cp_amm::cp_amm_create_pool::cp_amm_create_pool;
    
    fn main() {
        if let Err(e) = cp_amm_create_pool() {
            eprintln!("{}", e);
        }
    }
```

Example of output:
```text
Running `target/debug/rdm1`
🚀 Starting Raydium CP-AMM Liquidity Pool Creation on Devnet...
🔑 Creator Wallet: DzxWSmfP6AJTWtUgHWdkUHbFoMCMirD2nxchRaAbWmJM
- Token 0 Mint: 4JERHdTjMWSXYJd4tBuDohyNknYDLL5kWRJyGv9gY8bh
- Token 1 Mint: FpxYcEJBRUFJ46XAcoVRPNJhWnjEGzUY4rQgErEbnegr

🔍 Deriving PDAs...
- Authority PDA: 7rQ1QFNosMkUCuh7Z7fPbTHvh73b68sQYdirycEzJVuw
- AmmConfig PDA: 9zSzfkYy6awexsHvmggeH36pfVUdDGyCcwmjT3AQPBj6
- Pool State PDA: 549ozjy4M83ZXxvYNYk9qQgYrwX9FisYLb9JZsXdRWAf
- LP Mint PDA: 3ry7sMLrzKdcXCmK5SQDRdztEe1SxMUZz8fHKRK337z2
- Token 0 Vault PDA: 8mxrQjyRsg4KFAoLwwtSSnp8C5ZBJxKDECca9RAefAMK
- Token 1 Vault PDA: C7yYzfghJyUq3cmqYogfjPywdEeW681zHdujqa1qg4x7
- Observation State PDA: Cbg2pzcP39X8HGYsGYstjJqHZTUXibi5KzsHZMoito14

📋 Assembling Accounts for `initialize` instruction...
- Creator Token 0 ATA: 8FAozQVK2S5D6Sdm2Zp1uKj3khfjeteKbWEtv3hJX3pq
- Creator Token 1 ATA: 4rovNWExfAhxzwxuV4Et5rM26pg67tfDw27uXBUNPAfd

📦 Instruction Data Prepared:
- Initial Token 0 (lamports): 1000000000000
- Initial Token 1 (lamports): 1000000000000
- Open Time (Unix Timestamp): 1751423378

📡 Sending transaction to Solana Devnet...

✅ Transaction successful!
- Signature: 3dx3Y8pGWtzhsVttJdbVCSNReUPn3pLmFQWW7amWhmdnqVgidJs7Cw5f3Kp5jWK7HYjBpYXjYuMMqZWGruhbDMCt
- Solana Explorer: https://explorer.solana.com/tx/3dx3Y8pGWtzhsVttJdbVCSNReUPn3pLmFQWW7amWhmdnqVgidJs7Cw5f3Kp5jWK7HYjBpYXjYuMMqZWGruhbDMCt?cluster=devnet
```

2. **CP-AMM deposit `src/main.rs`**: [4Yso1Fh7ZXu3UtzJzBWRMuTSgyqU2Rro7EXGFKSDpuQ4SD9Np7nyG3PbN571s9ucAubYz4rS2uJ8UFTnB89waMNf](https://explorer.solana.com/tx/4Yso1Fh7ZXu3UtzJzBWRMuTSgyqU2Rro7EXGFKSDpuQ4SD9Np7nyG3PbN571s9ucAubYz4rS2uJ8UFTnB89waMNf?cluster=devnet)
```rust
    // src/main.rs

    use anyhow::Result;
    use solana_client::rpc_client::RpcClient;
    use solana_sdk::signature::Keypair;
    
    mod instructions;
    use instructions::cp_amm::cp_amm_deposit_liquidity::cp_amm_deposit_liquidity;
    
    fn main() -> Result<()> {
        let rpc_client = RpcClient::new("https://api.devnet.solana.com".to_string());
    
        let secret_key_json = r#"[86,238,130,90,23,141,232,132,110,230,236,214,227,119,72,63,117,103,243,211,223,26,222,234,246,236,177,248,136,216,158,11,193,37,28,168,115,125,97,184,5,54,12,59,136,67,70,60,55,200,9,122,232,119,247,226,62,130,155,50,83,164,207,166]"#;
        let secret_key: Vec<u8> = serde_json::from_str(secret_key_json)?;
        let user = Keypair::from_bytes(&secret_key)?;
    
        let lp_to_deposit = 500 * 1_000_000_000;
    
        println!("Attempting to deposit liquidity...");
    
        match cp_amm_deposit_liquidity(&rpc_client, &user, lp_to_deposit) {
            Ok(signature) => {
                println!("✅ Liquidity deposit successful!");
                println!("   Transaction Signature: {}", signature);
                println!(
                    "   Solana Explorer: https://explorer.solana.com/tx/{}?cluster=devnet",
                    signature
                );
            }
            Err(e) => {
                eprintln!("❌ Liquidity deposit failed: {}", e);
            }
        }
    
        Ok(())
    }
```

Example of output:
```text
Attempting to deposit liquidity...
Depositing 500000000000 LP tokens into pool 549ozjy4M83ZXxvYNYk9qQgYrwX9FisYLb9JZsXdRWAf
Fetching live pool data...
Required Token 0: 500000000000, Max Allowed: 505000000000
Required Token 1: 500000000000, Max Allowed: 505000000000
Sending deposit transaction...
✅ Liquidity deposit successful!
Transaction Signature: 4Yso1Fh7ZXu3UtzJzBWRMuTSgyqU2Rro7EXGFKSDpuQ4SD9Np7nyG3PbN571s9ucAubYz4rS2uJ8UFTnB89waMNf
Solana Explorer: https://explorer.solana.com/tx/4Yso1Fh7ZXu3UtzJzBWRMuTSgyqU2Rro7EXGFKSDpuQ4SD9Np7nyG3PbN571s9ucAubYz4rS2uJ8UFTnB89waMNf?cluster=devnet
```

3. **CP-AMM withdraw `src/main.rs`**: [2JmXjm7QXwc1JoJBBnAB8CeCw6AVQoQZqUQamn6K2qUy3cL7ZPv9CWtRtg1PYPeumY2UVXY3Aev4zitqCtvcJavf](https://explorer.solana.com/tx/2JmXjm7QXwc1JoJBBnAB8CeCw6AVQoQZqUQamn6K2qUy3cL7ZPv9CWtRtg1PYPeumY2UVXY3Aev4zitqCtvcJavf?cluster=devnet)
```rust
    // src/main.rs

    use anyhow::Result;
    use solana_client::rpc_client::RpcClient;
    use solana_sdk::signature::Keypair;
    
    mod instructions;
    use instructions::cp_amm::cp_amm_withdraw_liquidity::cp_amm_withdraw_liquidity;
    
    fn main() -> Result<()> {
        let rpc_client = RpcClient::new("https://api.devnet.solana.com".to_string());
    
        let secret_key_json = r#"[86,238,130,90,23,141,232,132,110,230,236,214,227,119,72,63,117,103,243,211,223,26,222,234,246,236,177,248,136,216,158,11,193,37,28,168,115,125,97,184,5,54,12,59,136,67,70,60,55,200,9,122,232,119,247,226,62,130,155,50,83,164,207,166]"#;
        let secret_key: Vec<u8> = serde_json::from_str(secret_key_json)?;
        let user = Keypair::from_bytes(&secret_key)?;
    
        let lp_to_withdraw = 300 * 1_000_000_000;
    
        println!("\nAttempting to withdraw liquidity...");
    
        match cp_amm_withdraw_liquidity(&rpc_client, &user, lp_to_withdraw) {
            Ok(signature) => {
                println!("✅ Liquidity withdrawal successful!");
                println!("   Transaction Signature: {}", signature);
                println!("   Solana Explorer: https://explorer.solana.com/tx/{}?cluster=devnet", signature);
            }
            Err(e) => {
                eprintln!("❌ Liquidity withdrawal failed: {}", e);
            }
        }
    
        Ok(())
    }
```

Example of output:

```text
Attempting to withdraw liquidity...
Withdrawing 300000000000 LP tokens from pool 549ozjy4M83ZXxvYNYk9qQgYrwX9FisYLb9JZsXdRWAf
Fetching live pool data...
Expected Token 0: 300000000000, Min Accepted: 297000000000
Expected Token 1: 300000000000, Min Accepted: 297000000000
Sending withdraw transaction...
✅ Liquidity withdrawal successful!
Transaction Signature: 2JmXjm7QXwc1JoJBBnAB8CeCw6AVQoQZqUQamn6K2qUy3cL7ZPv9CWtRtg1PYPeumY2UVXY3Aev4zitqCtvcJavf
Solana Explorer: https://explorer.solana.com/tx/2JmXjm7QXwc1JoJBBnAB8CeCw6AVQoQZqUQamn6K2qUy3cL7ZPv9CWtRtg1PYPeumY2UVXY3Aev4zitqCtvcJavf?cluster=devnet
```


4. **CP-AMM atomic deposit-withdraw  `src/main.rs`**: [2JxVsUgngJYoQwPzzUHAbnpke9qKyBCk2f8dCUP1yiPBLVyTYgjfDUPuGRT2yeWxoQakCvEbSCV2pqfTzZnPraSE](https://explorer.solana.com/tx/2JxVsUgngJYoQwPzzUHAbnpke9qKyBCk2f8dCUP1yiPBLVyTYgjfDUPuGRT2yeWxoQakCvEbSCV2pqfTzZnPraSE?cluster=devnet)
```rust
    // src/main.rs

    use anyhow::Result;
    use solana_client::rpc_client::RpcClient;
    use solana_sdk::signature::Keypair;
    
    mod instructions;
    use instructions::cp_amm::cp_amm_atomic_deposit_withdraw::cp_amm_atomic_deposit_then_withdraw;
    
    fn main() -> Result<()> {
        let rpc_client = RpcClient::new("https://api.devnet.solana.com".to_string());
    
        let secret_key_json = r#"[86,238,130,90,23,141,232,132,110,230,236,214,227,119,72,63,117,103,243,211,223,26,222,234,246,236,177,248,136,216,158,11,193,37,28,168,115,125,97,184,5,54,12,59,136,67,70,60,55,200,9,122,232,119,247,226,62,130,155,50,83,164,207,166]"#;
        let secret_key: Vec<u8> = serde_json::from_str(secret_key_json)?;
        let user = Keypair::from_bytes(&secret_key)?;
    
        let lp_token_amount = 100 * 1_000_000_000;
    
        println!("\nAttempting atomic deposit-then-withdraw...");
    
        match cp_amm_atomic_deposit_then_withdraw(&rpc_client, &user, lp_token_amount) {
            Ok(signature) => {
                println!("✅ Atomic deposit-then-withdraw successful!");
                println!("   Transaction Signature: {}", signature);
                println!(
                    "   Solana Explorer: https://explorer.solana.com/tx/{}?cluster=devnet",
                    signature
                );
                println!("   Check your token balances. They should be unchanged (minus fees).");
            }
            Err(e) => {
                eprintln!("❌ Atomic transaction failed: {}", e);
            }
        }
    
        Ok(())
    }
```

Example of output:

```text
Attempting atomic deposit-then-withdraw...
Building atomic deposit-then-withdraw transaction for 100000000000 LP tokens...
Sending atomic transaction...
✅ Atomic deposit-then-withdraw successful!
Transaction Signature: 2JxVsUgngJYoQwPzzUHAbnpke9qKyBCk2f8dCUP1yiPBLVyTYgjfDUPuGRT2yeWxoQakCvEbSCV2pqfTzZnPraSE
Solana Explorer: https://explorer.solana.com/tx/2JxVsUgngJYoQwPzzUHAbnpke9qKyBCk2f8dCUP1yiPBLVyTYgjfDUPuGRT2yeWxoQakCvEbSCV2pqfTzZnPraSE?cluster=devnet
```


5. **CP-AMM atomic withdraw-deposit  `src/main.rs`**: [QPBjC25tpKLniz386kCzhAWSp43w6TTrVGFsAHJnVun48BuMqaWZokXpf5dKe6Dx4HoAs4ucm79agbz2aezk71B](https://explorer.solana.com/tx/QPBjC25tpKLniz386kCzhAWSp43w6TTrVGFsAHJnVun48BuMqaWZokXpf5dKe6Dx4HoAs4ucm79agbz2aezk71B?cluster=devnet)
```rust
    // src/main.rs

    use anyhow::Result;
    use solana_client::rpc_client::RpcClient;
    use solana_sdk::signature::Keypair;
    
    mod instructions;
    use instructions::cp_amm::cp_amm_atomic_withdraw_deposit::cp_amm_atomic_withdraw_then_deposit;
    
    fn main() -> Result<()> {

        let rpc_client = RpcClient::new("https://api.devnet.solana.com".to_string());
    
        let secret_key_json = r#"[86,238,130,90,23,141,232,132,110,230,236,214,227,119,72,63,117,103,243,211,223,26,222,234,246,236,177,248,136,216,158,11,193,37,28,168,115,125,97,184,5,54,12,59,136,67,70,60,55,200,9,122,232,119,247,226,62,130,155,50,83,164,207,166]"#;
        let secret_key: Vec<u8> = serde_json::from_str(secret_key_json)?;
        let user = Keypair::from_bytes(&secret_key)?;
    
        let lp_token_amount = 100 * 1_000_000_000;
    
        println!("\nAttempting atomic withdraw-then-deposit...");
    
        match cp_amm_atomic_withdraw_then_deposit(&rpc_client, &user, lp_token_amount) {
            Ok(signature) => {
                println!("✅ Atomic withdraw-then-deposit successful!");
                println!("   Transaction Signature: {}", signature);
                println!("   Solana Explorer: https://explorer.solana.com/tx/{}?cluster=devnet", signature);
                println!("   As with the previous atomic transaction, your token balances should be unchanged.");
            }
            Err(e) => {
                eprintln!("❌ Atomic transaction failed: {}", e);
            }
        }
    
        Ok(())
    }
```

Example of output:

```text
Attempting atomic withdraw-then-deposit...
Building atomic withdraw-then-deposit transaction for 100000000000 LP tokens...
Sending atomic transaction...
✅ Atomic withdraw-then-deposit successful!
Transaction Signature: QPBjC25tpKLniz386kCzhAWSp43w6TTrVGFsAHJnVun48BuMqaWZokXpf5dKe6Dx4HoAs4ucm79agbz2aezk71B
Solana Explorer: https://explorer.solana.com/tx/QPBjC25tpKLniz386kCzhAWSp43w6TTrVGFsAHJnVun48BuMqaWZokXpf5dKe6Dx4HoAs4ucm79agbz2aezk71B?cluster=devnet
```


6. **CLMM create pool `src/main.rs`**: [3caCvTtpseAd8Efw3jVmX3CbNkdMJ5kj7Ge4AEFZJN9Fd5ytW8F8Lt9TB42Yw74vLKK69xTFVxMuq59i9azi3fby](https://explorer.solana.com/tx/3caCvTtpseAd8Efw3jVmX3CbNkdMJ5kj7Ge4AEFZJN9Fd5ytW8F8Lt9TB42Yw74vLKK69xTFVxMuq59i9azi3fby?cluster=devnet)
 ```rust
    // src/main.rs

    mod instructions;
    use instructions::clmm::clmm_create_pool::clmm_create_pool;

    fn main() {
        if let Err(e) =  clmm_create_pool() {
            eprintln!("Error: {}", e);
        }
    }
```

Example of output:

```text
Using wallet: DzxWSmfP6AJTWtUgHWdkUHbFoMCMirD2nxchRaAbWmJM
Derived Pool State PDA: SBdWWdRY7BrexhV6vxw4K8DAoe6ZNHmEzyhyRmKtdzY
Derived Token Vault 0 PDA: CQmgWjG7A2BEMy8ym6ZHucSvvBvrLknueF6KSF3SgFc9
Derived Token Vault 1 PDA: qL76AR9dgR8Tuomv2EpiFTw32t18YapbvK1kcsWeaXK
Derived Observation State PDA: 9TWKekwxbQYofMCLAAjnK3EkssdDgDvf9uq6WVgQX7Wc
Derived Tick Array Bitmap PDA: AZZHUcDCkFdRmmunTa7fk88RXGtf48Nm1GJsFfhCJa2N
Sending create_pool transaction...
Transaction successful with signature: 3caCvTtpseAd8Efw3jVmX3CbNkdMJ5kj7Ge4AEFZJN9Fd5ytW8F8Lt9TB42Yw74vLKK69xTFVxMuq59i9azi3fby
```


**Run from the command line**:

```bash
    cargo run
```

The program will print the transaction signature upon successful execution, which could be then looked up on the Solana Explorer for Devnet.
