use crate::constants::{ADMIN_CONFIG_SEED, ASSET_LOOKUP_TABLE_SEED, JLP_MINT, USDC_MINT};
use anchor_client::solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey};
use anchor_lang::{solana_program, AccountDeserialize};
use anchor_spl::token::TokenAccount;
use anyhow::{anyhow, Result};
use borsh::BorshDeserialize;
use clap::Parser;
use pyth_solana_receiver_sdk::price_update::PriceUpdateV2;
use solana_client::rpc_client::RpcClient;
use spl_associated_token_account::get_associated_token_address;
use std::str::FromStr;

mod constants;

const AUM_VALUE_SCALE_DECIMALS: u8 = 6;
const PROGRAM_ID: &str = "UtycozxZPRv91c2ibTA1pmvFoFqqVCAoZ1jxYSgArpM";

fn account_deserialize<T: BorshDeserialize>(data: &[u8]) -> Result<T> {
    if data.len() < 8 {
        return Err(anyhow!("Account data too short"));
    }
    let mut account_data: &[u8] = &data[8..];
    T::deserialize(&mut account_data).map_err(|e| anyhow!("Failed to deserialize account: {:?}", e))
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "https://api.mainnet-beta.solana.com")]
    url: String,
}

#[derive(BorshDeserialize, Debug)]
pub struct AssetLookupTable {
    pub asset_mint: Pubkey,
    pub oracle_account: Pubkey,
    pub decimals: u8,
    pub token_account_owners: Vec<Pubkey>,
}

#[derive(BorshDeserialize, Debug)]
pub struct UnitasConfig {
    pub admin: Pubkey,
    pub pending_admin: Pubkey,
    pub aum_usd: u128,
    pub last_updated_timestamp: i64,
    pub usdu_config: Pubkey,
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

fn calculate_asset_value(
    rpc_client: &RpcClient,
    asset_lookup_table: &AssetLookupTable,
) -> Result<u128> {
    let oracle_acc = rpc_client.get_account(&asset_lookup_table.oracle_account)?;
    let price_account: PriceUpdateV2 = PriceUpdateV2::try_deserialize(&mut &oracle_acc.data[..])
        .map_err(|e| anyhow!("Failed to deserialize price account: {:?}", e))?;

    let price = price_account.price_message.price;
    let price_value: u128 = price.abs() as u128;
    let price_decimals: u8 = price_account.price_message.exponent.abs() as u8;
    let token_decimals = asset_lookup_table.decimals;

    println!(
        "\n--- Calculating Value for Mint: {} ---",
        asset_lookup_table.asset_mint
    );
    println!("Raw price: {}", price_value);
    println!(
        "Actual price: {}",
        price_value as f64 / ten_pow(price_decimals) as f64
    );

    let token_accounts: Vec<(Pubkey, TokenAccount)> = asset_lookup_table
        .token_account_owners
        .iter()
        .filter_map(|owner| {
            let pubkey = get_associated_token_address(owner, &asset_lookup_table.asset_mint);
            match rpc_client.get_account(&pubkey) {
                Ok(account) => match TokenAccount::try_deserialize(&mut &account.data[..]) {
                    Ok(token_account) => Some((pubkey, token_account)),
                    Err(_) => {
                        println!("Warning: Failed to deserialize token account {}", pubkey);
                        None
                    }
                },
                Err(_) => None, // Token account does not exist or RPC error
            }
        })
        .collect();

    let mut total_asset_value: u128 = 0;
    for (ata_pubkey, token_account) in token_accounts.iter() {
        let token_amount: u128 = token_account.amount.into();
        println!(
            "\nProcessing Owner: {}, ATA: {}",
            token_account.owner, ata_pubkey
        );
        println!("Raw token amount: {}", token_amount);

        let raw_value = price_value * token_amount;
        let total_decimals = price_decimals + token_decimals;
        let asset_value_usd = if total_decimals > AUM_VALUE_SCALE_DECIMALS {
            let diff = total_decimals - AUM_VALUE_SCALE_DECIMALS;
            raw_value / ten_pow(diff)
        } else {
            let diff = AUM_VALUE_SCALE_DECIMALS - total_decimals;
            raw_value * ten_pow(diff)
        };
        println!("USD value (scaled): {}", asset_value_usd);
        total_asset_value += asset_value_usd;
    }

    println!("Total USD value for this asset: {}", total_asset_value);
    Ok(total_asset_value)
}

fn main() -> Result<()> {
    let args = Args::parse();
    let rpc_client = RpcClient::new_with_commitment(args.url, CommitmentConfig::confirmed());
    let program_id = Pubkey::from_str(PROGRAM_ID)?;

    // 1. Derive the global UnitasConfig PDA
    let (unitas_config_pda, _) =
        Pubkey::find_program_address(&[ADMIN_CONFIG_SEED.as_bytes()], &program_id);
    println!("Derived UnitasConfig PDA: {}", unitas_config_pda);

    let unitas_config_acc = rpc_client.get_account(&unitas_config_pda)?;
    let unitas_config = account_deserialize::<UnitasConfig>(&unitas_config_acc.data)?;

    println!(
        "Unitas Config Last Updated Timestamp: {}",
        unitas_config.last_updated_timestamp
    );

    // 2. Initialize total_value with the base AUM from the config
    let mut total_value: u128 = unitas_config.aum_usd;
    println!("Initial AUM from Config: {}", total_value);

    // 3. Iterate through the list of tracked asset mints
    let tracked_asset_mints = vec![Pubkey::from_str(JLP_MINT)?, Pubkey::from_str(USDC_MINT)?];

    for asset_mint in tracked_asset_mints {
        // 4. Derive and validate the AssetLookupTable PDA for each mint
        let (asset_lookup_table_pda, _) = Pubkey::find_program_address(
            &[ASSET_LOOKUP_TABLE_SEED.as_bytes(), asset_mint.as_ref()],
            &program_id,
        );

        println!(
            "\nDerived AssetLookupTable PDA for mint {}: {}",
            asset_mint, asset_lookup_table_pda
        );

        let lookup_table_acc_result = rpc_client.get_account(&asset_lookup_table_pda);
        if lookup_table_acc_result.is_err() {
            println!(
                "Warning: AssetLookupTable not found for mint {}. Skipping.",
                asset_mint
            );
            continue;
        }

        let lookup_table_acc = lookup_table_acc_result.unwrap();
        let asset_lookup_table = account_deserialize::<AssetLookupTable>(&lookup_table_acc.data)?;

        // 5. Perform the crucial validation
        if asset_lookup_table.asset_mint != asset_mint {
            return Err(anyhow!(
                "Validation failed! PDA {} does not contain the expected mint {}. Found {} instead.",
                asset_lookup_table_pda,
                asset_mint,
                asset_lookup_table.asset_mint
            ));
        }
        println!("PDA validation successful for mint {}", asset_mint);

        // 6. Calculate the value for this asset and add it to the total
        let asset_value = calculate_asset_value(&rpc_client, &asset_lookup_table)?;
        total_value += asset_value;
    }

    // 7. Fetch USDU total supply for price calculation
    let usdu_config_acc = rpc_client.get_account(&unitas_config.usdu_config)?;
    let usdu_config = account_deserialize::<UsduConfig>(&usdu_config_acc.data)?;

    println!("\n--- Final AUM Calculation ---");
    println!("Total AUM value: {}", total_value);
    println!(
        "Total AUM value (human readable): {}",
        total_value as f64 / ten_pow(AUM_VALUE_SCALE_DECIMALS) as f64
    );
    println!("USDU total supply: {}", usdu_config.total_supply);
    println!(
        "USDU price: {}",
        total_value as f64 / usdu_config.total_supply as f64
    );

    Ok(())
}
