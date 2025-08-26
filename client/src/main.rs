use std::str::FromStr;
use anchor_lang::AccountDeserialize;
use anchor_client::solana_sdk::{
    pubkey::Pubkey,
    commitment_config::CommitmentConfig,
};
use borsh::BorshDeserialize;
use anyhow::{Result, anyhow};
use clap::Parser;
use solana_client::rpc_client::RpcClient;
use pyth_solana_receiver_sdk::price_update::PriceUpdateV2;
use anchor_spl::token::TokenAccount;
use spl_associated_token_account::get_associated_token_address;

const AUM_VALUE_SCALE_DECIMALS: u8 = 6;

// Account addresses
const USDU_CONFIG: &str = "om3x6puF7Beqxc1WYPCYBWwUZMZ77hYk7AsMEbi8Fez";
const LOOKUP_TABLE: &str = "EckQZfk9n8g22Edmh2vcRGLHrvabfCZ63AVZe8jzxvMM";

const JLP_ORACLE: &str = "2TTGSRSezqF7Beqxc1WYPCYBWwUZMZ77hYk7AsMEbi8Fez";
const USDC_ORACLE: &str = "Dpw1EAVrSB1ibxiDQyTAW6Zip3J4Btk2x4SgApQCeFbX";

// Mint addresses
const USDC_MINT: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
const JLP_MINT: &str = "27G8MtK7VtTcCHkpASjSDdkWWYfoqT6ggEuKidVJidD4";

fn account_deserialize<T: BorshDeserialize>(data: &[u8]) -> Result<T> {
    if data.len() < 8 {
        return Err(anyhow!("Account data too short"));
    }
    let mut account_data: &[u8] = &data[8..];
    T::deserialize(&mut account_data)
        .map_err(|e| anyhow!("Failed to deserialize account: {:?}", e))
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "https://api.mainnet-beta.solana.com")]
    url: String,
}

#[derive(BorshDeserialize, Debug)]
pub struct AssetLookupTable {
    pub aum_usd: u128,
    pub last_updated_timestamp: i64,
    pub jlp_oracle_account: Pubkey,
    pub usdc_oracle_account: Pubkey,
    pub usdc_mint: Pubkey,
    pub jlp_mint: Pubkey,
    pub usdu_config: Pubkey,
    pub usdc_token_account_owner: Pubkey,
    pub jlp_token_account_owners: Vec<Pubkey>,
}

#[derive(BorshDeserialize, Debug)]
pub struct UsduConfig {
    pub admin: Pubkey,
    pub pending_admin: Pubkey,
    pub access_registry: Pubkey,
    pub bump: u8,
    pub is_initialized: bool,

    pub usdu_token: Pubkey,
    pub usdu_token_bump: u8,
    pub is_usdu_token_initialized: bool,

    pub total_supply: u128,
}

pub fn ten_pow(exponent: impl Into<u32>) -> u128 {
    let expo = exponent.into();
    let value: u128 = match expo {
        30 => 1_000_000_000_000_000_000_000_000_000_000,
        29 => 100_000_000_000_000_000_000_000_000_000,
        28 => 10_000_000_000_000_000_000_000_000_000,
        27 => 1_000_000_000_000_000_000_000_000_000,
        26 => 100_000_000_000_000_000_000_000_000,
        25 => 10_000_000_000_000_000_000_000_000,
        24 => 1_000_000_000_000_000_000_000_000,
        23 => 100_000_000_000_000_000_000_000,
        22 => 10_000_000_000_000_000_000_000,
        21 => 1_000_000_000_000_000_000_000,
        20 => 100_000_000_000_000_000_000,
        19 => 10_000_000_000_000_000_000,
        18 => 1_000_000_000_000_000_000,
        17 => 100_000_000_000_000_000,
        16 => 10_000_000_000_000_000,
        15 => 1_000_000_000_000_000,
        14 => 100_000_000_000_000,
        13 => 10_000_000_000_000,
        12 => 1_000_000_000_000,
        11 => 100_000_000_000,
        10 => 10_000_000_000,
        9 => 1_000_000_000,
        8 => 100_000_000,
        7 => 10_000_000,
        6 => 1_000_000,
        5 => 100_000,
        4 => 10_000,
        3 => 1_000,
        2 => 100,
        1 => 10,
        0 => 1,
        _ => panic!("no support for exponent: {expo}"),
    };

    value
}

fn calculate_jlp_value(
    rpc_client: &RpcClient,
    jlp_accounts: &[(Pubkey, TokenAccount)],
) -> Result<u128> {
    let jlp_oracle_pubkey = Pubkey::from_str(JLP_ORACLE)?;
    let jlp_oracle_acc = rpc_client.get_account(&jlp_oracle_pubkey)?;

    let jlp_price_account: PriceUpdateV2 =
        PriceUpdateV2::try_deserialize(&mut &jlp_oracle_acc.data[..])
            .map_err(|e| anyhow!("Failed to deserialize price account: {:?}", e))?;
    let jlp_price = jlp_price_account.price_message.price;
    let jlp_price_value: u128 = jlp_price.abs() as u128;
    let jlp_price_decimals: u8 = jlp_price_account.price_message.exponent.abs() as u8;
    let jlp_token_decimals = 6;
    println!("JLP raw price: {}", jlp_price_value);
    println!(
        "JLP actual price: {}",
        jlp_price_value as f64 / ten_pow(jlp_price_decimals) as f64
    );
    println!("JLP price decimals: {}", jlp_price_decimals);
    println!("JLP token decimals: {}", jlp_token_decimals);

    let mut accrue_value: u128 = 0;
    for (jlp_token_account_pubkey, token_account) in jlp_accounts.iter() {
        println!("\nProcessing JLP account {}", jlp_token_account_pubkey);
        let token_amount: u128 = token_account.amount.into();
        println!("Raw token amount: {}", token_amount);
        println!(
            "Actual token amount: {}",
            token_amount as f64 / ten_pow(jlp_token_decimals) as f64
        );

        let raw_value = jlp_price_value * token_amount;
        println!("Raw multiplication result: {}", raw_value);

        let total_decimals = jlp_price_decimals + jlp_token_decimals;
        let token_amount_usd = if total_decimals > AUM_VALUE_SCALE_DECIMALS {
            let diff = total_decimals - AUM_VALUE_SCALE_DECIMALS;
            raw_value / ten_pow(diff)
        } else {
            let diff = AUM_VALUE_SCALE_DECIMALS - total_decimals;
            raw_value * ten_pow(diff)
        };

        println!("Token USD value: {}", token_amount_usd);
        println!(
            "Token USD value (human readable): {}",
            token_amount_usd as f64 / ten_pow(AUM_VALUE_SCALE_DECIMALS) as f64
        );
        accrue_value += token_amount_usd;
    }
    println!("\n");
    println!("JLP accrue value: {}", accrue_value);
    println!(
        "JLP accrue value (human readable): {}",
        accrue_value as f64 / ten_pow(AUM_VALUE_SCALE_DECIMALS) as f64
    );

    Ok(accrue_value)
}

fn calculate_usdc_value(rpc_client: &RpcClient, lookup_table: &AssetLookupTable) -> Result<u128> {
    let usdc_token_account_pubkey = get_associated_token_address(
        &lookup_table.usdc_token_account_owner,
        &lookup_table.usdc_mint
    );
    let usdc_account = rpc_client.get_account(&usdc_token_account_pubkey)?;
    let usdc_token_account = TokenAccount::try_deserialize(&mut &usdc_account.data[..])
        .map_err(|e| anyhow!("Failed to deserialize token account {}: {:?}", usdc_token_account_pubkey, e))?;
    let usdc_amount: u128 = usdc_token_account.amount.into();
    let usdc_oracle_pubkey = Pubkey::from_str(USDC_ORACLE)?;
    let usdc_oracle_acc = rpc_client.get_account(&usdc_oracle_pubkey)?;
    let usdc_price_account: PriceUpdateV2 = PriceUpdateV2::try_deserialize(&mut &usdc_oracle_acc.data[..])
        .map_err(|e| anyhow!("Failed to deserialize price account: {:?}", e))?;
    let usdc_price = usdc_price_account.price_message.price;
    let usdc_price_value: u128 = usdc_price.abs() as u128;
    let usdc_price_decimals: u8 = usdc_price_account.price_message.exponent.abs() as u8;
    let usdc_token_decimals: u8 = 6;
    println!("\nProcessing USDC account {}", usdc_token_account_pubkey);
    println!("USDC raw token amount: {}", usdc_amount);
    println!("USDC actual token amount: {}", usdc_amount as f64 / ten_pow(usdc_token_decimals) as f64);
    println!("USDC raw price: {}", usdc_price_value);
    let raw_value = usdc_price_value * usdc_amount;
    let total_decimals = usdc_price_decimals + usdc_token_decimals;
    let usdc_value = if total_decimals > AUM_VALUE_SCALE_DECIMALS {
        let diff = total_decimals - AUM_VALUE_SCALE_DECIMALS;
        raw_value / ten_pow(diff)
    } else {
        let diff = AUM_VALUE_SCALE_DECIMALS - total_decimals;
        raw_value * ten_pow(diff)
    };
    println!("Usdc value: {}", usdc_value);
    println!("Usdc value (human readable): {}", usdc_value as f64 / ten_pow(AUM_VALUE_SCALE_DECIMALS) as f64);
    Ok(usdc_value)
}

fn check_all_pubkeys(lookup_table: &AssetLookupTable) -> Result<()> {
    let jlp_oracle_pubkey = Pubkey::from_str(JLP_ORACLE)?;
    let usdc_oracle_pubkey = Pubkey::from_str(USDC_ORACLE)?;
    let usdc_mint_pubkey = Pubkey::from_str(USDC_MINT)?;
    let jlp_mint_pubkey = Pubkey::from_str(JLP_MINT)?;
    let usdu_config_pubkey = Pubkey::from_str(USDU_CONFIG)?;

    assert_eq!(lookup_table.jlp_oracle_account, jlp_oracle_pubkey);
    assert_eq!(lookup_table.usdc_oracle_account, usdc_oracle_pubkey);
    assert_eq!(lookup_table.usdc_mint, usdc_mint_pubkey);
    assert_eq!(lookup_table.jlp_mint, jlp_mint_pubkey);
    assert_eq!(lookup_table.usdu_config, usdu_config_pubkey);
    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();
    let rpc_client = RpcClient::new_with_commitment(args.url, CommitmentConfig::confirmed());
    
    let lookup_table_pubkey = Pubkey::from_str(LOOKUP_TABLE)?;
    let lookup_table_acc = rpc_client.get_account(&lookup_table_pubkey)?;
    let lookup_table = account_deserialize::<AssetLookupTable>(&lookup_table_acc.data)
        .map_err(|e| anyhow!("Failed to deserialize lookup table: {:?}, data length: {}", e, lookup_table_acc.data.len()))?;

    println!("Lookup table last updated timestamp: {}", lookup_table.last_updated_timestamp);
    check_all_pubkeys(&lookup_table)?;
    
    let usdu_config_pubkey = Pubkey::from_str(USDU_CONFIG)?;
    let usdu_config_acc = rpc_client.get_account(&usdu_config_pubkey)?;
    let usdu_config = account_deserialize::<UsduConfig>(&usdu_config_acc.data)
        .map_err(|e| anyhow!("Failed to deserialize USDU config: {:?}, data length: {}", e, usdu_config_acc.data.len()))?;
    
    let jlp_accounts: Vec<(Pubkey, TokenAccount)> = lookup_table
        .jlp_token_account_owners
        .iter()
        .map(|owner| {
            let pubkey = get_associated_token_address(owner, &lookup_table.jlp_mint);
            let account = rpc_client.get_account(&pubkey).unwrap();
            let token_account = TokenAccount::try_deserialize(&mut &account.data[..])
                .map_err(|e| {
                    anyhow!(
                        "Failed to deserialize token account {}: {:?}",
                        pubkey,
                        e
                    )
                })
                .unwrap();
            (pubkey, token_account)
        })
        .collect();
    
    let mut total_value: u128 = lookup_table.aum_usd;
    println!("Initial AUM: {}", total_value);

    let jlp_accrue_value = calculate_jlp_value(&rpc_client, &jlp_accounts)?;
    total_value += jlp_accrue_value;

    let usdc_value = calculate_usdc_value(&rpc_client, &lookup_table)?;
    total_value += usdc_value;

    println!("\nTotal AUM value: {}", total_value);
    println!("Total AUM value (human readable): {}", total_value as f64 / ten_pow(AUM_VALUE_SCALE_DECIMALS) as f64);
    println!("USDU total supply: {}", usdu_config.total_supply);
    println!("USDU total supply (human readable): {}", usdu_config.total_supply as f64 / ten_pow(AUM_VALUE_SCALE_DECIMALS) as f64);
    println!("USDU price: {}", total_value as f64 / usdu_config.total_supply as f64);
    
    Ok(())
}