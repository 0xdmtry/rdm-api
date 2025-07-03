use anyhow::{format_err, Result};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    program_pack::Pack,
    pubkey::Pubkey,
    signature::{Keypair, Signature, Signer},
    transaction::Transaction,
};
use spl_token::state::Account as TokenAccount;
use std::str::FromStr;

#[derive(BorshSerialize, BorshDeserialize, Debug)]
struct DepositInstructionData {
    pub lp_token_amount: u64,
    pub maximum_token_0_amount: u64,
    pub maximum_token_1_amount: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
struct WithdrawInstructionData {
    pub lp_token_amount: u64,
    pub minimum_token_0_amount: u64,
    pub minimum_token_1_amount: u64,
}

#[derive(BorshDeserialize, Debug, Clone)]
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
            vault_0_balance.checked_sub(self.protocol_fees_token_0 + self.fund_fees_token_0).unwrap(),
            vault_1_balance.checked_sub(self.protocol_fees_token_1 + self.fund_fees_token_1).unwrap(),
        )
    }
}

mod curve_calculator {
    #[derive(Debug)]
    pub struct TradingTokenResult { pub token_0_amount: u128, pub token_1_amount: u128 }
    #[derive(PartialEq, Eq)]
    pub enum RoundDirection { Floor, Ceiling }
    pub fn lp_tokens_to_trading_tokens(lp_token_amount: u128, lp_token_supply: u128, swap_token_0_amount: u128, swap_token_1_amount: u128, round_direction: RoundDirection) -> Option<TradingTokenResult> {
        if lp_token_supply == 0 { return None; }
        let mut token_0_amount = lp_token_amount.checked_mul(swap_token_0_amount)?.checked_div(lp_token_supply)?;
        let mut token_1_amount = lp_token_amount.checked_mul(swap_token_1_amount)?.checked_div(lp_token_supply)?;
        if round_direction == RoundDirection::Ceiling {
            if lp_token_amount.checked_mul(swap_token_0_amount)?.checked_rem(lp_token_supply)? > 0 && token_0_amount > 0 { token_0_amount = token_0_amount.checked_add(1)?; }
            if lp_token_amount.checked_mul(swap_token_1_amount)?.checked_rem(lp_token_supply)? > 0 && token_1_amount > 0 { token_1_amount = token_1_amount.checked_add(1)?; }
        }
        Some(TradingTokenResult { token_0_amount, token_1_amount })
    }
}

pub fn cp_amm_atomic_deposit_then_withdraw(
    rpc_client: &RpcClient,
    user: &Keypair,
    lp_token_amount: u64,
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

    println!("Building atomic deposit-then-withdraw transaction for {} LP tokens...", lp_token_amount);

    let accounts_to_fetch = vec![pool_id, token_0_vault, token_1_vault];
    let mut account_data = rpc_client.get_multiple_accounts(&accounts_to_fetch)?;
    let pool_state_data = account_data.remove(0).ok_or_else(|| format_err!("Pool state not found"))?.data;
    let pool_state = PoolState::try_from_slice(&pool_state_data[8..])?;
    let token_0_vault_state = TokenAccount::unpack(&account_data.remove(0).ok_or_else(|| format_err!("Vault 0 not found"))?.data)?;
    let token_1_vault_state = TokenAccount::unpack(&account_data.remove(0).ok_or_else(|| format_err!("Vault 1 not found"))?.data)?;
    let (pool_token_0_balance, pool_token_1_balance) = pool_state.vault_amount_without_fee(token_0_vault_state.amount, token_1_vault_state.amount);

    let deposit_ix = {
        let required_tokens = curve_calculator::lp_tokens_to_trading_tokens(lp_token_amount as u128, pool_state.lp_supply as u128, pool_token_0_balance as u128, pool_token_1_balance as u128, curve_calculator::RoundDirection::Ceiling).ok_or_else(|| format_err!("Calc failed"))?;
        let maximum_token_0_amount = required_tokens.token_0_amount as u64 * 101 / 100;
        let maximum_token_1_amount = required_tokens.token_1_amount as u64 * 101 / 100;
        let instruction_data = DepositInstructionData { lp_token_amount, maximum_token_0_amount, maximum_token_1_amount };
        let mut data_with_discriminator = Vec::with_capacity(32);
        data_with_discriminator.extend_from_slice(&[242, 35, 198, 137, 82, 225, 242, 182]);
        data_with_discriminator.extend_from_slice(&instruction_data.try_to_vec()?);
        let accounts = vec![ AccountMeta::new_readonly(user.pubkey(), true), AccountMeta::new_readonly(pool_authority, false), AccountMeta::new(pool_id, false), AccountMeta::new(user_lp_token_ata, false), AccountMeta::new(user_token_0_ata, false), AccountMeta::new(user_token_1_ata, false), AccountMeta::new(token_0_vault, false), AccountMeta::new(token_1_vault, false), AccountMeta::new_readonly(spl_token::id(), false), AccountMeta::new_readonly(Pubkey::from_str("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb")?, false), AccountMeta::new_readonly(token_0_mint, false), AccountMeta::new_readonly(token_1_mint, false), AccountMeta::new(lp_mint, false)];
        Instruction { program_id, accounts, data: data_with_discriminator }
    };

    let withdraw_ix = {
        let future_lp_supply = pool_state.lp_supply.checked_add(lp_token_amount).ok_or_else(|| format_err!("LP supply overflow"))?;
        let expected_tokens = curve_calculator::lp_tokens_to_trading_tokens(lp_token_amount as u128, future_lp_supply as u128, pool_token_0_balance as u128, pool_token_1_balance as u128, curve_calculator::RoundDirection::Floor).ok_or_else(|| format_err!("Calc failed"))?;
        let minimum_token_0_amount = expected_tokens.token_0_amount as u64 * 99 / 100;
        let minimum_token_1_amount = expected_tokens.token_1_amount as u64 * 99 / 100;
        let instruction_data = WithdrawInstructionData { lp_token_amount, minimum_token_0_amount, minimum_token_1_amount };
        let mut data_with_discriminator = Vec::with_capacity(32);
        data_with_discriminator.extend_from_slice(&[183, 18, 70, 156, 148, 109, 161, 34]);
        data_with_discriminator.extend_from_slice(&instruction_data.try_to_vec()?);
        let accounts = vec![ AccountMeta::new_readonly(user.pubkey(), true), AccountMeta::new_readonly(pool_authority, false), AccountMeta::new(pool_id, false), AccountMeta::new(user_lp_token_ata, false), AccountMeta::new(user_token_0_ata, false), AccountMeta::new(user_token_1_ata, false), AccountMeta::new(token_0_vault, false), AccountMeta::new(token_1_vault, false), AccountMeta::new_readonly(spl_token::id(), false), AccountMeta::new_readonly(Pubkey::from_str("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb")?, false), AccountMeta::new_readonly(token_0_mint, false), AccountMeta::new_readonly(token_1_mint, false), AccountMeta::new(lp_mint, false), AccountMeta::new_readonly(spl_memo::id(), false)];
        Instruction { program_id, accounts, data: data_with_discriminator }
    };

    let latest_blockhash = rpc_client.get_latest_blockhash()?;
    let transaction = Transaction::new_signed_with_payer(
        &[deposit_ix, withdraw_ix],
        Some(&user.pubkey()),
        &[user],
        latest_blockhash,
    );

    println!("Sending atomic transaction...");
    rpc_client.send_and_confirm_transaction_with_spinner(&transaction).map_err(anyhow::Error::from)
}
