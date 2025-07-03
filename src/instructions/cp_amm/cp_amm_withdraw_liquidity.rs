use anyhow::{Result, format_err};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    program_pack::Pack,
    pubkey::Pubkey,
    signature::{Keypair, Signature, Signer},
    transaction::Transaction,
};
use spl_memo;
use spl_token::state::Account as TokenAccount;
use std::str::FromStr;

#[derive(BorshSerialize, BorshDeserialize, Debug)]
struct WithdrawInstructionData {
    pub lp_token_amount: u64,
    pub minimum_token_0_amount: u64,
    pub minimum_token_1_amount: u64,
}

#[derive(BorshDeserialize, Debug)]
#[allow(dead_code)]
struct PoolState {
    pub amm_config: Pubkey,
    pub pool_creator: Pubkey,
    pub token_0_vault: Pubkey,
    pub token_1_vault: Pubkey,
    pub lp_mint: Pubkey,
    pub token_0_mint: Pubkey,
    pub token_1_mint: Pubkey,
    pub token_0_program: Pubkey,
    pub token_1_program: Pubkey,
    pub observation_key: Pubkey,
    pub auth_bump: u8,
    pub status: u8,
    pub lp_mint_decimals: u8,
    pub mint_0_decimals: u8,
    pub mint_1_decimals: u8,
    pub lp_supply: u64,
    pub protocol_fees_token_0: u64,
    pub protocol_fees_token_1: u64,
    pub fund_fees_token_0: u64,
    pub fund_fees_token_1: u64,
    pub open_time: u64,
    pub recent_epoch: u64,
    pub padding: [u64; 31],
}

impl PoolState {
    fn vault_amount_without_fee(&self, vault_0_balance: u64, vault_1_balance: u64) -> (u64, u64) {
        (
            vault_0_balance
                .checked_sub(self.protocol_fees_token_0 + self.fund_fees_token_0)
                .unwrap(),
            vault_1_balance
                .checked_sub(self.protocol_fees_token_1 + self.fund_fees_token_1)
                .unwrap(),
        )
    }
}

mod curve_calculator {
    #[derive(Debug)]
    pub struct TradingTokenResult {
        pub token_0_amount: u128,
        pub token_1_amount: u128,
    }

    #[derive(PartialEq, Eq)]
    pub enum RoundDirection {
        Floor,
        Ceiling,
    }

    pub fn lp_tokens_to_trading_tokens(
        lp_token_amount: u128,
        lp_token_supply: u128,
        swap_token_0_amount: u128,
        swap_token_1_amount: u128,
        round_direction: RoundDirection,
    ) -> Option<TradingTokenResult> {
        if lp_token_supply == 0 {
            return None;
        }
        let token_0_amount = lp_token_amount
            .checked_mul(swap_token_0_amount)?
            .checked_div(lp_token_supply)?;
        let token_1_amount = lp_token_amount
            .checked_mul(swap_token_1_amount)?
            .checked_div(lp_token_supply)?;

        Some(TradingTokenResult {
            token_0_amount,
            token_1_amount,
        })
    }
}

pub fn cp_amm_withdraw_liquidity(
    rpc_client: &RpcClient,
    user: &Keypair,
    lp_token_amount_to_withdraw: u64,
) -> Result<Signature> {
    let program_id = Pubkey::from_str("CPMDWBwJDtYax9qW7AyRuVC19Cc4L4Vcy4n2BHAbHkCW")?;
    let pool_id = Pubkey::from_str("549ozjy4M83ZXxvYNYk9qQgYrwX9FisYLb9JZsXdRWAf")?;
    let pool_authority = Pubkey::from_str("7rQ1QFNosMkUCuh7Z7fPbTHvh73b68sQYdirycEzJVuw")?;
    let lp_mint = Pubkey::from_str("3ry7sMLrzKdcXCmK5SQDRdztEe1SxMUZz8fHKRK337z2")?;
    let token_0_mint = Pubkey::from_str("4JERHdTjMWSXYJd4tBuDohyNknYDLL5kWRJyGv9gY8bh")?;
    let token_1_mint = Pubkey::from_str("FpxYcEJBRUFJ46XAcoVRPNJhWnjEGzUY4rQgErEbnegr")?;
    let token_0_vault = Pubkey::from_str("8mxrQjyRsg4KFAoLwwtSSnp8C5ZBJxKDECca9RAefAMK")?;
    let token_1_vault = Pubkey::from_str("C7yYzfghJyUq3cmqYogfjPywdEeW681zHdujqa1qg4x7")?;

    let user_token_0_ata = Pubkey::from_str("8FAozQVK2S5D6Sdm2Zp1uKj3khfjeteKbWEtv3hJX3pq")?;
    let user_token_1_ata = Pubkey::from_str("4rovNWExfAhxzwxuV4Et5rM26pg67tfDw27uXBUNPAfd")?;
    let user_lp_token_ata = Pubkey::from_str("CiekUybiVWscUvuqdwXQXdzpYcjBsq8FNTFyHx4hPdyy")?;

    println!(
        "Withdrawing {} LP tokens from pool {}",
        lp_token_amount_to_withdraw, pool_id
    );

    println!("Fetching live pool data...");
    let accounts_to_fetch = vec![pool_id, token_0_vault, token_1_vault];
    let mut account_data = rpc_client.get_multiple_accounts(&accounts_to_fetch)?;

    let pool_state_data = account_data
        .remove(0)
        .ok_or_else(|| format_err!("Pool state account not found"))?
        .data;
    let pool_state = PoolState::try_from_slice(&pool_state_data[8..])?;

    let token_0_vault_data = account_data
        .remove(0)
        .ok_or_else(|| format_err!("Token 0 vault not found"))?
        .data;
    let token_0_vault_state = TokenAccount::unpack(&token_0_vault_data)?;

    let token_1_vault_data = account_data
        .remove(0)
        .ok_or_else(|| format_err!("Token 1 vault not found"))?
        .data;
    let token_1_vault_state = TokenAccount::unpack(&token_1_vault_data)?;

    let (pool_token_0_balance, pool_token_1_balance) =
        pool_state.vault_amount_without_fee(token_0_vault_state.amount, token_1_vault_state.amount);

    let expected_tokens = curve_calculator::lp_tokens_to_trading_tokens(
        lp_token_amount_to_withdraw as u128,
        pool_state.lp_supply as u128,
        pool_token_0_balance as u128,
        pool_token_1_balance as u128,
        curve_calculator::RoundDirection::Floor,
    )
    .ok_or_else(|| format_err!("Failed to calculate expected tokens"))?;

    let token_0_to_receive = expected_tokens.token_0_amount as u64;
    let token_1_to_receive = expected_tokens.token_1_amount as u64;

    const SLIPPAGE_BPS: u64 = 100;
    let minimum_token_0_amount = token_0_to_receive - (token_0_to_receive * SLIPPAGE_BPS / 10000);
    let minimum_token_1_amount = token_1_to_receive - (token_1_to_receive * SLIPPAGE_BPS / 10000);

    println!(
        "Expected Token 0: {}, Min Accepted: {}",
        token_0_to_receive, minimum_token_0_amount
    );
    println!(
        "Expected Token 1: {}, Min Accepted: {}",
        token_1_to_receive, minimum_token_1_amount
    );

    let instruction_discriminator: [u8; 8] = [183, 18, 70, 156, 148, 109, 161, 34];

    let instruction_data = WithdrawInstructionData {
        lp_token_amount: lp_token_amount_to_withdraw,
        minimum_token_0_amount,
        minimum_token_1_amount,
    };

    let mut data_with_discriminator = Vec::with_capacity(8 + 24);
    data_with_discriminator.extend_from_slice(&instruction_discriminator);
    data_with_discriminator.extend_from_slice(&instruction_data.try_to_vec()?);

    let accounts = vec![
        AccountMeta::new_readonly(user.pubkey(), true),
        AccountMeta::new_readonly(pool_authority, false),
        AccountMeta::new(pool_id, false),
        AccountMeta::new(user_lp_token_ata, false),
        AccountMeta::new(user_token_0_ata, false),
        AccountMeta::new(user_token_1_ata, false),
        AccountMeta::new(token_0_vault, false),
        AccountMeta::new(token_1_vault, false),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(
            Pubkey::from_str("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb")?,
            false,
        ),
        AccountMeta::new_readonly(token_0_mint, false),
        AccountMeta::new_readonly(token_1_mint, false),
        AccountMeta::new(lp_mint, false),
        AccountMeta::new_readonly(spl_memo::id(), false),
    ];

    let instruction = Instruction {
        program_id,
        accounts,
        data: data_with_discriminator,
    };

    let latest_blockhash = rpc_client.get_latest_blockhash()?;
    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&user.pubkey()),
        &[user],
        latest_blockhash,
    );

    println!("Sending withdraw transaction...");
    let signature = rpc_client.send_and_confirm_transaction_with_spinner(&transaction)?;

    Ok(signature)
}
