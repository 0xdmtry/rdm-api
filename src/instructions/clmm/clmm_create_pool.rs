use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    pubkey::Pubkey,
    signer::{Signer, keypair::Keypair},
    system_instruction,
    transaction::Transaction,
};
use spl_token;
use std::error::Error;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

const RAYDIUM_CLMM_PROGRAM_ID: &str = "devi51mZmdwUJGU9hjN27vEz64Gps7uUefqxg27EAtH";
const AMM_CONFIG_ID: &str = "CQYbhr6amxUER4p5SC44C63R4qw4NFc9Z4Db9vF4tZwG";

const TOKEN_MINT_0_ADDR: &str = "4JERHdTjMWSXYJd4tBuDohyNknYDLL5kWRJyGv9gY8bh";
const TOKEN_MINT_1_ADDR: &str = "FpxYcEJBRUFJ46XAcoVRPNJhWnjEGzUY4rQgErEbnegr";

pub fn create_clmm_liquidity_pool() -> Result<(), Box<dyn Error>> {
    println!("Raydium Devnet Liquidity Pool Creator");
    println!("------------------------------------");

    let rpc_client = RpcClient::new("https://api.devnet.solana.com".to_string());

    let secret_key_string = "[86,238,130,90,23,141,232,132,110,230,236,214,227,119,72,63,117,103,243,211,223,26,222,234,246,236,177,248,136,216,158,11,193,37,28,168,115,125,97,184,5,54,12,59,136,67,70,60,55,200,9,122,232,119,247,226,62,130,155,50,83,164,207,166]";
    let secret_key: Vec<u8> =
        serde_json::from_str(secret_key_string).expect("Failed to parse secret key.");

    if secret_key.len() != 64 {
        panic!(
            "Secret key must be 64 bytes long, but got {}",
            secret_key.len()
        );
    }
    let pool_creator =
        Keypair::from_bytes(&secret_key).expect("Failed to create keypair from secret key bytes.");

    println!("Using wallet: {}", pool_creator.pubkey());

    let clmm_program_id = Pubkey::from_str(RAYDIUM_CLMM_PROGRAM_ID).map_err(|e| {
        format!(
            "ERROR: Failed to parse RAYDIUM_CLMM_PROGRAM_ID ({}): {}",
            RAYDIUM_CLMM_PROGRAM_ID, e
        )
    })?;

    let amm_config_id = Pubkey::from_str(AMM_CONFIG_ID).map_err(|e| {
        format!(
            "ERROR: Failed to parse AMM_CONFIG_ID ({}): {}",
            AMM_CONFIG_ID, e
        )
    })?;

    let token_mint_0 = Pubkey::from_str(TOKEN_MINT_0_ADDR).map_err(|e| {
        format!(
            "ERROR: Failed to parse TOKEN_MINT_0_ADDR ({}): {}",
            TOKEN_MINT_0_ADDR, e
        )
    })?;

    let token_mint_1 = Pubkey::from_str(TOKEN_MINT_1_ADDR).map_err(|e| {
        format!(
            "ERROR: Failed to parse TOKEN_MINT_1_ADDR ({}): {}",
            TOKEN_MINT_1_ADDR, e
        )
    })?;

    if token_mint_0 >= token_mint_1 {
        panic!("Error: token_mint_0 must be strictly less than token_mint_1. Please swap them.");
    }

    let (pool_state_pda, _pool_bump) = Pubkey::find_program_address(
        &[
            b"pool",
            amm_config_id.as_ref(),
            token_mint_0.as_ref(),
            token_mint_1.as_ref(),
        ],
        &clmm_program_id,
    );
    println!("Derived Pool State PDA: {}", pool_state_pda);

    let (token_vault_0_pda, _vault_0_bump) = Pubkey::find_program_address(
        &[
            b"pool_vault",
            pool_state_pda.as_ref(),
            token_mint_0.as_ref(),
        ],
        &clmm_program_id,
    );
    println!("Derived Token Vault 0 PDA: {}", token_vault_0_pda);

    let (token_vault_1_pda, _vault_1_bump) = Pubkey::find_program_address(
        &[
            b"pool_vault",
            pool_state_pda.as_ref(),
            token_mint_1.as_ref(),
        ],
        &clmm_program_id,
    );
    println!("Derived Token Vault 1 PDA: {}", token_vault_1_pda);

    let (observation_pda, _observation_bump) =
        Pubkey::find_program_address(&[b"observation", pool_state_pda.as_ref()], &clmm_program_id);
    println!("Derived Observation State PDA: {}", observation_pda);

    let (tick_array_bitmap_pda, _bitmap_bump) = Pubkey::find_program_address(
        &[b"pool_tick_array_bitmap_extension", pool_state_pda.as_ref()],
        &clmm_program_id,
    );
    println!("Derived Tick Array Bitmap PDA: {}", tick_array_bitmap_pda);

    create_pool(
        &rpc_client,
        &pool_creator,
        &clmm_program_id,
        &amm_config_id,
        &pool_state_pda,
        &token_mint_0,
        &token_mint_1,
        &token_vault_0_pda,
        &token_vault_1_pda,
        &observation_pda,
        &tick_array_bitmap_pda,
    )?;

    println!("\nPool successfully created!");
    Ok(())
}

fn create_pool(
    rpc_client: &RpcClient,
    pool_creator: &Keypair,
    clmm_program_id: &Pubkey,
    amm_config: &Pubkey,
    pool_state: &Pubkey,
    token_mint_0: &Pubkey,
    token_mint_1: &Pubkey,
    token_vault_0: &Pubkey,
    token_vault_1: &Pubkey,
    observation_state: &Pubkey,
    tick_array_bitmap: &Pubkey,
) -> Result<(), Box<dyn Error>> {
    let instruction_discriminator: [u8; 8] = [0xe9, 0x92, 0xd1, 0x8e, 0xcf, 0x68, 0x40, 0xbc];

    let initial_sqrt_price: u128 = 7530851732716320752100;

    let open_time: u64 = 0;

    let mut create_pool_instruction_data = Vec::new();
    create_pool_instruction_data.extend_from_slice(&instruction_discriminator);
    create_pool_instruction_data.extend_from_slice(&initial_sqrt_price.to_le_bytes());
    create_pool_instruction_data.extend_from_slice(&open_time.to_le_bytes());

    let create_pool_accounts = vec![
        solana_sdk::instruction::AccountMeta::new(pool_creator.pubkey(), true),
        solana_sdk::instruction::AccountMeta::new_readonly(*amm_config, false),
        solana_sdk::instruction::AccountMeta::new(*pool_state, false),
        solana_sdk::instruction::AccountMeta::new_readonly(*token_mint_0, false),
        solana_sdk::instruction::AccountMeta::new_readonly(*token_mint_1, false),
        solana_sdk::instruction::AccountMeta::new(*token_vault_0, false),
        solana_sdk::instruction::AccountMeta::new(*token_vault_1, false),
        solana_sdk::instruction::AccountMeta::new(*observation_state, false),
        solana_sdk::instruction::AccountMeta::new(*tick_array_bitmap, false),
        solana_sdk::instruction::AccountMeta::new_readonly(spl_token::id(), false),
        solana_sdk::instruction::AccountMeta::new_readonly(spl_token::id(), false),
        solana_sdk::instruction::AccountMeta::new_readonly(solana_sdk::system_program::id(), false),
        solana_sdk::instruction::AccountMeta::new_readonly(solana_sdk::sysvar::rent::id(), false),
    ];

    let create_pool_ix = solana_sdk::instruction::Instruction {
        program_id: *clmm_program_id,
        accounts: create_pool_accounts,
        data: create_pool_instruction_data,
    };

    let latest_blockhash = rpc_client.get_latest_blockhash()?;
    let transaction = Transaction::new_signed_with_payer(
        &[create_pool_ix],
        Some(&pool_creator.pubkey()),
        &[pool_creator],
        latest_blockhash,
    );

    println!("Sending create_pool transaction...");
    let signature = rpc_client.send_and_confirm_transaction_with_spinner(&transaction)?;

    println!("Transaction successful with signature: {}", signature);
    Ok(())
}
