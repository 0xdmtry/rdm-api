use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey};

pub async fn fetch_accounts_and_signatures(pid: &str, rpc_url: &str) {
    let client = RpcClient::new_with_commitment(rpc_url.to_string(), CommitmentConfig::confirmed());

    let program_id: Pubkey = pid.parse().expect("Invalid program ID");

    println!("Fetching accounts owned by Raydium AMM program...");

    match client.get_program_accounts(&program_id).await {
        Ok(accounts) => {
            println!("Found {} accounts", accounts.len());

            for (i, (pubkey, _account)) in accounts.iter().take(10).enumerate() {
                println!("\n[{}] Account: {}", i + 1, pubkey);

                match client.get_signatures_for_address(pubkey).await {
                    Ok(sigs) => {
                        if sigs.is_empty() {
                            println!("    No recent transactions found");
                        } else {
                            for sig in sigs.iter().take(3) {
                                println!("    Tx: {}", sig.signature);
                            }
                        }
                    }
                    Err(e) => {
                        println!("    Error fetching signatures: {}", e);
                    }
                }
            }
        }
        Err(err) => {
            eprintln!("Error fetching program accounts: {:?}", err);
        }
    }
}
