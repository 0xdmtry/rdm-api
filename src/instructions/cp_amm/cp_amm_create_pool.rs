use anyhow::{Result, format_err};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_program, sysvar,
    transaction::Transaction,
};
use spl_associated_token_account::get_associated_token_address;
use std::str::FromStr;
use std::time::SystemTime;

pub fn cp_amm_create_pool() -> Result<()> {

    const RAYDIUM_CP_SWAP_PROGRAM_ID: &str = "CPMDWBwJDtYax9qW7AyRuVC19Cc4L4Vcy4n2BHAbHkCW";
    const CREATOR_SECRET_KEY_JSON: &str = r#"[86,238,130,90,23,141,232,132,110,230,236,214,227,119,72,63,117,103,243,211,223,26,222,234,246,236,177,248,136,216,158,11,193,37,28,168,115,125,97,184,5,54,12,59,136,67,70,60,55,200,9,122,232,119,247,226,62,130,155,50,83,164,207,166]"#;

    const TOKEN_A_MINT_STR: &str = "4JERHdTjMWSXYJd4tBuDohyNknYDLL5kWRJyGv9gY8bh";
    const TOKEN_B_MINT_STR: &str = "FpxYcEJBRUFJ46XAcoVRPNJhWnjEGzUY4rQgErEbnegr";

    const INITIAL_TOKEN_A_AMOUNT: u64 = 1_000_000_000_000;
    const INITIAL_TOKEN_B_AMOUNT: u64 = 1_000_000_000_000;

    const CREATE_POOL_FEE_RECEIVER_ID: &str = "G11FKBRaAkHAKuLCgLM6K6NUc9rTjPAznRCjZifrTQe2";

    const AUTH_SEED: &[u8] = b"vault_and_lp_mint_auth_seed";
    const AMM_CONFIG_SEED: &[u8] = b"amm_config";
    const POOL_SEED: &[u8] = b"pool";
    const POOL_LP_MINT_SEED: &[u8] = b"pool_lp_mint";
    const POOL_VAULT_SEED: &[u8] = b"pool_vault";
    const OBSERVATION_SEED: &[u8] = b"observation";

    #[derive(BorshSerialize, BorshDeserialize, Debug)]
    pub struct InitializeInstructionData {
        pub init_amount_0: u64,
        pub init_amount_1: u64,
        pub open_time: u64,
    }

    println!("🚀 Starting Raydium CP-AMM Liquidity Pool Creation on Devnet...");

    let rpc_client = RpcClient::new("https://api.devnet.solana.com".to_string());
    let program_id = Pubkey::from_str(RAYDIUM_CP_SWAP_PROGRAM_ID)?;

    let secret_key: Vec<u8> = serde_json::from_str(CREATOR_SECRET_KEY_JSON)?;
    let creator = Keypair::from_bytes(&secret_key)
        .map_err(|e| format_err!("Failed to create keypair from secret key bytes: {}", e))?;
    println!("🔑 Creator Wallet: {}", creator.pubkey());

    let token_a_mint = Pubkey::from_str(TOKEN_A_MINT_STR)?;
    let token_b_mint = Pubkey::from_str(TOKEN_B_MINT_STR)?;

    let (token_0_mint, token_1_mint, init_amount_0, init_amount_1) = if token_a_mint < token_b_mint
    {
        (
            token_a_mint,
            token_b_mint,
            INITIAL_TOKEN_A_AMOUNT,
            INITIAL_TOKEN_B_AMOUNT,
        )
    } else {
        (
            token_b_mint,
            token_a_mint,
            INITIAL_TOKEN_B_AMOUNT,
            INITIAL_TOKEN_A_AMOUNT,
        )
    };

    println!("   - Token 0 Mint: {}", token_0_mint);
    println!("   - Token 1 Mint: {}", token_1_mint);

    println!("\n🔍 Deriving PDAs...");

    let (authority_pda, _) = Pubkey::find_program_address(&[AUTH_SEED], &program_id);
    println!("   - Authority PDA: {}", authority_pda);

    let amm_config_index = 0u16;
    let (amm_config_pda, _) = Pubkey::find_program_address(
        &[AMM_CONFIG_SEED, &amm_config_index.to_le_bytes()],
        &program_id,
    );
    println!("   - AmmConfig PDA: {}", amm_config_pda);

    let (pool_state_pda, _) = Pubkey::find_program_address(
        &[
            POOL_SEED,
            amm_config_pda.as_ref(),
            token_0_mint.as_ref(),
            token_1_mint.as_ref(),
        ],
        &program_id,
    );
    println!("   - Pool State PDA: {}", pool_state_pda);

    let (lp_mint_pda, _) =
        Pubkey::find_program_address(&[POOL_LP_MINT_SEED, pool_state_pda.as_ref()], &program_id);
    println!("   - LP Mint PDA: {}", lp_mint_pda);

    let (token_0_vault_pda, _) = Pubkey::find_program_address(
        &[
            POOL_VAULT_SEED,
            pool_state_pda.as_ref(),
            token_0_mint.as_ref(),
        ],
        &program_id,
    );
    println!("   - Token 0 Vault PDA: {}", token_0_vault_pda);

    let (token_1_vault_pda, _) = Pubkey::find_program_address(
        &[
            POOL_VAULT_SEED,
            pool_state_pda.as_ref(),
            token_1_mint.as_ref(),
        ],
        &program_id,
    );
    println!("   - Token 1 Vault PDA: {}", token_1_vault_pda);

    let (observation_state_pda, _) =
        Pubkey::find_program_address(&[OBSERVATION_SEED, pool_state_pda.as_ref()], &program_id);
    println!("   - Observation State PDA: {}", observation_state_pda);

    println!("\n📋 Assembling Accounts for `initialize` instruction...");

    let creator_token_0_ata = get_associated_token_address(&creator.pubkey(), &token_0_mint);
    let creator_token_1_ata = get_associated_token_address(&creator.pubkey(), &token_1_mint);
    let creator_lp_token_ata = get_associated_token_address(&creator.pubkey(), &lp_mint_pda);

    println!("   - Creator Token 0 ATA: {}", creator_token_0_ata);
    println!("   - Creator Token 1 ATA: {}", creator_token_1_ata);

    let create_pool_fee_pubkey = Pubkey::from_str(CREATE_POOL_FEE_RECEIVER_ID)?;

    let accounts = vec![
        AccountMeta::new(creator.pubkey(), true),
        AccountMeta::new_readonly(amm_config_pda, false),
        AccountMeta::new_readonly(authority_pda, false),
        AccountMeta::new(pool_state_pda, false),
        AccountMeta::new_readonly(token_0_mint, false),
        AccountMeta::new_readonly(token_1_mint, false),
        AccountMeta::new(lp_mint_pda, false),
        AccountMeta::new(creator_token_0_ata, false),
        AccountMeta::new(creator_token_1_ata, false),
        AccountMeta::new(creator_lp_token_ata, false),
        AccountMeta::new(token_0_vault_pda, false),
        AccountMeta::new(token_1_vault_pda, false),
        AccountMeta::new(create_pool_fee_pubkey, false),
        AccountMeta::new(observation_state_pda, false),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(spl_associated_token_account::id(), false),
        AccountMeta::new_readonly(system_program::id(), false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
    ];

    let open_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_secs();

    let instruction_data = InitializeInstructionData {
        init_amount_0,
        init_amount_1,
        open_time,
    };

    let mut data_with_discriminator = Vec::with_capacity(8 + 24);
    data_with_discriminator.extend_from_slice(&[175, 175, 109, 31, 13, 152, 155, 237]);
    data_with_discriminator.extend_from_slice(&instruction_data.try_to_vec()?);

    println!("\n📦 Instruction Data Prepared:");
    println!("   - Initial Token 0 (lamports): {}", init_amount_0);
    println!("   - Initial Token 1 (lamports): {}", init_amount_1);
    println!("   - Open Time (Unix Timestamp): {}", open_time);

    let instruction = Instruction {
        program_id,
        accounts,
        data: data_with_discriminator,
    };

    let latest_blockhash = rpc_client.get_latest_blockhash()?;
    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&creator.pubkey()),
        &[&creator],
        latest_blockhash,
    );

    println!("\n📡 Sending transaction to Solana Devnet...");
    match rpc_client.send_and_confirm_transaction_with_spinner(&transaction) {
        Ok(signature) => {
            println!("\n✅ Transaction successful!");
            println!("   - Signature: {}", signature);
            println!(
                "   - View on Solana Explorer: https://explorer.solana.com/tx/{}?cluster=devnet",
                signature
            );
        }
        Err(e) => {
            println!("\n❌ Transaction failed: {}", e);
        }
    }

    Ok(())
}
