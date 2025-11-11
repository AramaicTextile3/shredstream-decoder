use borsh::{BorshDeserialize, BorshSerialize};
use json::{object, JsonValue};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::pubkey;
use crate::utils::create_standardized_instruction;

pub const RAYDIUM_LAUNCHLAB_INITIALIZE_INSTRUCTION_DISCRIMINATOR: [u8; 8] = [175, 175, 109, 31, 13, 152, 155, 237];

pub const RAYDIUM_LAUNCHLAB_PROGRAM_ID: Pubkey = pubkey!("LanMV9sAd7wArD4vJFi2qDdfnVhFxYSUg6eADduJ3uj");
pub const SYSTEM_PROGRAM_ID: Pubkey = pubkey!("11111111111111111111111111111111");
pub const TOKEN_PROGRAM_ID: Pubkey = pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
pub const METADATA_PROGRAM_ID: Pubkey = pubkey!("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s");
pub const RENT_PROGRAM_ID: Pubkey = pubkey!("SysvarRent111111111111111111111111111111111");
pub const EVENT_AUTHORITY: Pubkey = pubkey!("2DPAtwB8L12vrMRExbLuyGnC7n2J5LNoZQSejeQGpwkr");
pub const RAYDIUM_LAUNCH_AUTHORITY: Pubkey = pubkey!("WLHv2UAZm6z4KyaaELi5pjdbJh6RESMva1Rnn8pJVVh");
pub const QUOTE_MINT: Pubkey = pubkey!("So11111111111111111111111111111111111111112");
pub const GLOBAL_CONFIG: Pubkey = pubkey!("6s1xP3hpbAfFoNtUNF8mfHsjr2Bd97JxFJRWLbL6aHuX");

pub enum RaydiumLaunchlabInstructionType {
    Initialize,
}

#[derive(BorshDeserialize, BorshSerialize, Debug, Default, Clone)]
pub struct MintParams {
    pub decimals: u8,
    pub name: String,
    pub symbol: String,
    pub uri: String,
}

#[derive(BorshDeserialize, BorshSerialize, Debug, Clone)]
pub enum CurveParams {
    Constant {
        supply: u64,
        total_base_sell: u64,
        total_quote_fund_raising: u64,
        migrate_type: u8,
    },
    Fixed {
        supply: u64,
        total_quote_fund_raising: u64,
        migrate_type: u8,
    },
    Linear {
        supply: u64,
        total_quote_fund_raising: u64,
        migrate_type: u8,
    },
}

impl Default for CurveParams {
    fn default() -> Self {
        CurveParams::Constant {
            supply: 0,
            total_base_sell: 0,
            total_quote_fund_raising: 0,
            migrate_type: 0,
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize, Debug, Default, Clone)]
pub struct VestingParams {
    pub total_locked_amount: u64,
    pub cliff_period: u64,
    pub unlock_period: u64,
}

pub fn get_raydium_launchlab_instruction_type(data: &[u8]) -> Option<RaydiumLaunchlabInstructionType> {
    match data.get(0..8) {
        Some(d) => {
            if d == RAYDIUM_LAUNCHLAB_INITIALIZE_INSTRUCTION_DISCRIMINATOR {
                Some(RaydiumLaunchlabInstructionType::Initialize)
            } else {
                None
            }
        },
        _ => None,
    }
}

pub fn deserialize_raydium_launchlab_initialize_instruction(data: &[u8], accounts_indices: &[u8], account_keys: &[Pubkey], is_signer: &[bool], is_writable: &[bool]) -> Result<JsonValue, String> {
    if data.len() < RAYDIUM_LAUNCHLAB_INITIALIZE_INSTRUCTION_DISCRIMINATOR.len() {
        return Err("Data length is insufficient for an 'initialize' instruction.".to_string());
    }
    
    let safely_get_account = |idx: usize| -> String {
        if idx >= accounts_indices.len() {
            return "Unknown".to_string();
        }
        
        let account_idx = accounts_indices[idx] as usize;
        if account_idx >= account_keys.len() {
            return "Unknown".to_string();
        }
        
        bs58::encode(&account_keys[account_idx]).into_string()
    };

    let remaining_data = &data[8..];
    let mut remaining_data_ref = &remaining_data[..];

    let base_mint_param = match MintParams::deserialize(&mut remaining_data_ref) {
        Ok(parsed) => parsed,
        Err(e) => {
            return Err(format!("Failed to deserialize MintParams: {:?}", e));
        }
    };

    let curve_param = match CurveParams::deserialize(&mut remaining_data_ref) {
        Ok(parsed) => parsed,
        Err(e) => {
            return Err(format!("Failed to deserialize CurveParams: {:?}", e));
        }
    };

    let vesting_param = match VestingParams::deserialize(&mut remaining_data_ref) {
        Ok(parsed) => parsed,
        Err(e) => {
            return Err(format!("Failed to deserialize VestingParams: {:?}", e));
        }
    };

    let accounts_json = object! {
        "creator" => safely_get_account(1),
        "globalConfig" => bs58::encode(GLOBAL_CONFIG).into_string(),
        "platformConfig" => safely_get_account(3),
        "authority" => bs58::encode(RAYDIUM_LAUNCH_AUTHORITY).into_string(),
        "poolState" => safely_get_account(5),
        "baseMint" => safely_get_account(6),
        "quoteMint" => bs58::encode(QUOTE_MINT).into_string(),
        "baseVault" => safely_get_account(8),
        "quoteVault" => safely_get_account(9),
        "baseTokenProgram" => bs58::encode(TOKEN_PROGRAM_ID).into_string(),
        "quoteTokenProgram" => bs58::encode(TOKEN_PROGRAM_ID).into_string(),
        "metadataProgram" => bs58::encode(METADATA_PROGRAM_ID).into_string(),
        "systemProgram" => bs58::encode(SYSTEM_PROGRAM_ID).into_string(),
        "rentProgram" => bs58::encode(RENT_PROGRAM_ID).into_string(),
        "eventAuthority" => bs58::encode(EVENT_AUTHORITY).into_string(),
        "program" => bs58::encode(RAYDIUM_LAUNCHLAB_PROGRAM_ID).into_string(),
    };

    let curve_json = match curve_param {
        CurveParams::Constant { supply, total_base_sell, total_quote_fund_raising, migrate_type } => {
            object! {
                "type" => "Constant",
                "supply" => supply,
                "totalBaseSell" => total_base_sell,
                "totalQuoteFundRaising" => total_quote_fund_raising,
                "migrateType" => migrate_type,
            }
        },
        CurveParams::Fixed { supply, total_quote_fund_raising, migrate_type } => {
            object! {
                "type" => "Fixed",
                "supply" => supply,
                "totalQuoteFundRaising" => total_quote_fund_raising,
                "migrateType" => migrate_type,
            }
        },
        CurveParams::Linear { supply, total_quote_fund_raising, migrate_type } => {
            object! {
                "type" => "Linear",
                "supply" => supply,
                "totalQuoteFundRaising" => total_quote_fund_raising,
                "migrateType" => migrate_type,
            }
        },
    };

    let parsed_data = object! {
        "baseMintParam" => object! {
            "decimals" => base_mint_param.decimals,
            "name" => base_mint_param.name,
            "symbol" => base_mint_param.symbol,
            "uri" => base_mint_param.uri,
        },
        "curveParam" => curve_json,
        "vestingParam" => object! {
            "totalLockedAmount" => vesting_param.total_locked_amount,
            "cliffPeriod" => vesting_param.cliff_period,
            "unlockPeriod" => vesting_param.unlock_period,
        },
    };

    Ok(create_standardized_instruction(
        &RAYDIUM_LAUNCHLAB_PROGRAM_ID,
        "Initialize",
        "RaydiumLaunchLab",
        data,
        accounts_indices,
        account_keys,
        is_signer,
        is_writable,
        parsed_data
    ))
}
