use std::str::FromStr;
use anchor_lang::AccountDeserialize;
use anchor_client::solana_sdk::{
    pubkey::Pubkey,
    commitment_config::CommitmentConfig,
};
use borsh::BorshDeserialize;
use anyhow::{Result, anyhow};
use clap::Parser;
use pyth_sdk_solana::state::{load_price_account, GenericPriceAccount};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{account::Account, clock::Clock};
use anchor_spl::token::TokenAccount;

const AUM_VALUE_SCALE_DECIMALS: u8 = 6;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// RPC URL
    #[arg(short, long, default_value = "https://api.mainnet-beta.solana.com")]
    url: String,

    /// Unitas Asset Lookup Table account
    #[arg(short, long)]
    lookup_table: String,

    /// JLP mint account
    #[arg(short, long)]
    mint: String,

    /// JLP Pyth oracle account
    #[arg(short, long)]
    oracle: String,

    /// USDU config account
    #[arg(short, long)]
    usdu_config: String,

    /// JLP token accounts (comma separated)
    #[arg(short, long)]
    accounts: String,
}

#[derive(BorshDeserialize, Debug)]
pub struct AssetLookupTable {
    pub index: u8,
    pub aum_usd: u128,
    pub mint: Pubkey,
    pub decimals: u8,
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
    
    let lookup_table_pubkey = Pubkey::from_str(&args.lookup_table)?;
    let lookup_table_acc = rpc_client.get_account(&lookup_table_pubkey)?;
    let lookup_table = AssetLookupTable::deserialize(&mut &lookup_table_acc.data[..])?;
    
    let mint_pubkey = Pubkey::from_str(&args.mint)?;
    let mint_acc = rpc_client.get_account(&mint_pubkey)?;
    
    let oracle_pubkey = Pubkey::from_str(&args.oracle)?;
    let oracle_acc = rpc_client.get_account(&oracle_pubkey)?;
    
    let usdu_config_pubkey = Pubkey::from_str(&args.usdu_config)?;
    let usdu_config_acc = rpc_client.get_account(&usdu_config_pubkey)?;
    let usdu_config = UsduConfig::deserialize(&mut &usdu_config_acc.data[..])?;
    
    // 获取 JLP token accounts
    let jlp_accounts: Vec<Account> = args.accounts
        .split(',')
        .map(|addr| {
            let pubkey = Pubkey::from_str(addr).unwrap();
            rpc_client.get_account(&pubkey).unwrap()
        })
        .collect();
    
    check_accounts(&lookup_table, &mint_acc, &jlp_accounts)?;
    
    let price_feed: &GenericPriceAccount<1, i64> = load_price_account(&oracle_acc.data)
        .map_err(|e| anyhow!("Failed to load price feed: {:?}", e))?;
    let price = price_feed.agg.price;
    let price_value: u128 = price.abs() as u128;
    let price_decimals: u8 = price_feed.expo.abs() as u8;
    let token_decimals = lookup_table.decimals;
    
    // 计算总价值
    let mut total_value: u128 = lookup_table.aum_usd;
    for jlp_acc in &jlp_accounts {
        let token_account = TokenAccount::try_deserialize(&mut &jlp_acc.data[..])?;
        let token_amount: u128 = token_account.amount.into();
        
        let token_amount_usd = if price_decimals + token_decimals > AUM_VALUE_SCALE_DECIMALS {
            let diff = price_decimals + token_decimals - AUM_VALUE_SCALE_DECIMALS;
            let nom = price_value * token_amount;
            let denom = ten_pow(u32::from(diff));
            nom / denom
        } else {
            let diff = AUM_VALUE_SCALE_DECIMALS - (price_decimals + token_decimals);
            price_value * token_amount * ten_pow(u32::from(diff))
        };
        total_value += token_amount_usd;
    }
    
    println!("Total AUM value: {}", total_value);
    println!("USDU total supply: {}", usdu_config.total_supply);
    println!("USDU price: {}", total_value as f64 / usdu_config.total_supply as f64);
    
    Ok(())
}

fn check_accounts(
    lookup_table: &AssetLookupTable,
    mint_acc: &Account,
    jlp_accounts: &[Account],
) -> Result<()> {
    if jlp_accounts.len() != lookup_table.accounts.len() {
        return Err(anyhow!("Account length mismatch"));
    }

    if lookup_table.mint != mint_acc.owner {
        return Err(anyhow!("Mint account mismatch"));
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
