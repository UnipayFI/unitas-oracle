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

const AUM_VALUE_SCALE_DECIMALS: u8 = 6;

// Account addresses
const LOOKUP_TABLE: &str = "DbdBYqBWyU9zywRSXT32frPuL3AhpaF2PQK7H42TpahS";
const MINT: &str = "27G8MtK7VtTcCHkpASjSDdkWWYfoqT6ggEuKidVJidD4";
const ORACLE: &str = "2TTGSRSezqFzeLUH8JwRUbtN66XLLaymfYsWRTMjfiMw";
const USDU_CONFIG: &str = "om3x6puF7Beqxc1WYPCYBWwUZMZ77hYk7AsMEbi8Fez";
const JLP_ACCOUNTS: &[&str] = &[
    "BC4MGsLxETeusWSJ17dnkWDS9eH23qT1yxwSjspvfVoB", // owner: 5ZbLoA6DSnXoDeU7jsdmmkua4X1ugHUFYzbByzrbJDST
    "3T8Tzwt4CvMJDbGH3Q9BVEyWofwA8cpjj7JRdGjktZXc", // owner: 8Qo4oKTM5jiZEAKzhBLKwTKjCJrDHsUUux5K5DaQDxLR
    "7aQWrYapnwLoPfGDa4ZobMk7xCcsx45hfz4EPgv9Jyj3", // owner: AR2ZCCyB5nXb7TesCz2pcCWbQsH8TAwixetDRrm3Z9wr
    "HwS956w2Whc77WgQRPxBxoo7Yd8ThJM4BjXh7vjBuTsH", // owner: EfMD9jVUnAkYeXv9fMaqC8rD4mc8dyVypFaR6DY9aHPs
];

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
    pub index: u8,
    pub aum_usd: u128,
    pub mint: Pubkey,
    pub decimals: u8,
    pub last_updated_timestamp: i64,
    pub accounts: Vec<Pubkey>,
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

fn main() -> Result<()> {
    let args = Args::parse();
    let rpc_client = RpcClient::new_with_commitment(args.url, CommitmentConfig::confirmed());
    
    let lookup_table_pubkey = Pubkey::from_str(LOOKUP_TABLE)?;
    let lookup_table_acc = rpc_client.get_account(&lookup_table_pubkey)?;
    let lookup_table = account_deserialize::<AssetLookupTable>(&lookup_table_acc.data)
        .map_err(|e| anyhow!("Failed to deserialize lookup table: {:?}, data length: {}", e, lookup_table_acc.data.len()))?;

    println!("Lookup table last updated timestamp: {}", lookup_table.last_updated_timestamp);
    
    let mint_pubkey = Pubkey::from_str(MINT)?;
    
    let oracle_pubkey = Pubkey::from_str(ORACLE)?;
    let oracle_acc = rpc_client.get_account(&oracle_pubkey)?;
    
    let usdu_config_pubkey = Pubkey::from_str(USDU_CONFIG)?;
    let usdu_config_acc = rpc_client.get_account(&usdu_config_pubkey)?;
    let usdu_config = account_deserialize::<UsduConfig>(&usdu_config_acc.data)
        .map_err(|e| anyhow!("Failed to deserialize USDU config: {:?}, data length: {}", e, usdu_config_acc.data.len()))?;
    
    let jlp_accounts: Vec<TokenAccount> = JLP_ACCOUNTS
        .iter()
        .map(|addr| {
            let pubkey = Pubkey::from_str(addr).unwrap();
            let account = rpc_client.get_account(&pubkey).unwrap();
            TokenAccount::try_deserialize(&mut &account.data[..])
                .map_err(|e| anyhow!("Failed to deserialize token account {}: {:?}", addr, e))
                .unwrap()
        })
        .collect();
    
    check_accounts(&lookup_table, &mint_pubkey, &jlp_accounts)?;
    
    let price_account: PriceUpdateV2 = PriceUpdateV2::try_deserialize(&mut &oracle_acc.data[..])
        .map_err(|e| anyhow!("Failed to deserialize price account: {:?}", e))?;
    let price = price_account.price_message.price;
    let price_value: u128 = price.abs() as u128;
    let price_decimals: u8 = price_account.price_message.exponent.abs() as u8;
    let token_decimals = lookup_table.decimals;
    println!("Raw price: {}", price_value);
    println!("Actual price: {}", price_value as f64 / ten_pow(price_decimals) as f64);
    println!("Price decimals: {}", price_decimals);
    println!("Token decimals: {}", token_decimals);
    
    // 计算总价值
    let mut total_value: u128 = lookup_table.aum_usd;
    let mut accrue_value: u128 = 0;
    println!("Initial AUM: {}", total_value);
    for (i, token_account) in jlp_accounts.iter().enumerate() {
        println!("\nProcessing JLP account {}", JLP_ACCOUNTS[i]);
        let token_amount: u128 = token_account.amount.into();
        println!("Raw token amount: {}", token_amount);
        println!("Actual token amount: {}", token_amount as f64 / ten_pow(token_decimals) as f64);
        
        // 直接计算实际值
        let raw_value = price_value * token_amount;
        println!("Raw multiplication result: {}", raw_value);
        
        // 调整到目标精度 (AUM_VALUE_SCALE_DECIMALS)
        let total_decimals = price_decimals + token_decimals;
        let token_amount_usd = if total_decimals > AUM_VALUE_SCALE_DECIMALS {
            let diff = total_decimals - AUM_VALUE_SCALE_DECIMALS;
            raw_value / ten_pow(diff)
        } else {
            let diff = AUM_VALUE_SCALE_DECIMALS - total_decimals;
            raw_value * ten_pow(diff)
        };
        
        println!("Token USD value: {}", token_amount_usd);
        println!("Token USD value (human readable): {}", token_amount_usd as f64 / ten_pow(AUM_VALUE_SCALE_DECIMALS) as f64);
        accrue_value += token_amount_usd;
    }

    println!("Accrue value: {}", accrue_value);
    println!("Accrue value (human readable): {}", accrue_value as f64 / ten_pow(AUM_VALUE_SCALE_DECIMALS) as f64);
    
    total_value += accrue_value;
    println!("\nTotal AUM value: {}", total_value);
    println!("Total AUM value (human readable): {}", total_value as f64 / ten_pow(AUM_VALUE_SCALE_DECIMALS) as f64);
    println!("USDU total supply: {}", usdu_config.total_supply);
    println!("USDU total supply (human readable): {}", usdu_config.total_supply as f64 / ten_pow(AUM_VALUE_SCALE_DECIMALS) as f64);
    println!("USDU price: {}", total_value as f64 / usdu_config.total_supply as f64);
    
    Ok(())
}

fn check_accounts(
    lookup_table: &AssetLookupTable,
    mint_pubkey: &Pubkey,
    jlp_accounts: &[TokenAccount],
) -> Result<()> {
    if jlp_accounts.len() != lookup_table.accounts.len() {
        return Err(anyhow!("Account length mismatch"));
    }

    if lookup_table.mint != *mint_pubkey {
        return Err(anyhow!("Mint account address mismatch"));
    }

    let mut expected_jlp_pks = lookup_table.accounts.clone();
    expected_jlp_pks.sort();

    let mut actual_owners: Vec<Pubkey> = jlp_accounts.iter()
        .map(|acc| acc.owner)
        .collect();
    actual_owners.sort();

    for (expected_pk, actual_owner) in expected_jlp_pks.iter().zip(actual_owners.iter()) {
        if expected_pk != actual_owner {
            return Err(anyhow!("Account owner mismatch"));
        }
    }

    Ok(())
}
