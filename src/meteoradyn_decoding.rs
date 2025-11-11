use borsh::BorshDeserialize;
use json::{object, JsonValue};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::pubkey;
use crate::utils::create_standardized_instruction;

pub const METEORADYN_PROGRAM_ID: Pubkey = pubkey!("Eo7WjKq67rjJQSZxS6z3YkapzY3eMj6Xy8X5EQVn5UaB");

pub const SYSTEM_PROGRAM_ID: Pubkey = pubkey!("11111111111111111111111111111111");
pub const TOKEN_PROGRAM_ID: Pubkey = pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
pub const ASSOCIATED_TOKEN_PROGRAM_ID: Pubkey = pubkey!("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");
pub const RENT_PROGRAM_ID: Pubkey = pubkey!("SysvarRent111111111111111111111111111111111");
pub const METADATA_PROGRAM_ID: Pubkey = pubkey!("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s");

const INIT_PERMISSIONLESS_POOL_IX_DISCRIMINATOR: [u8; 8] = [66, 5, 221, 69, 105, 8, 127, 249];

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum MeteoraDynInstructionType {
    InitializePermissionlessPool,
    Unknown,
}

#[derive(Debug, BorshDeserialize)]
pub enum CurveType {
    ConstantProduct,
    Stable { 
        amp: u64,
        token_multiplier: TokenMultiplier,
        depeg: Depeg,
        last_amp_updated_timestamp: u64
    }
}

#[derive(Debug, BorshDeserialize)]
pub struct TokenMultiplier {
    pub token_a_multiplier: u64,
    pub token_b_multiplier: u64,
    pub precision_factor: u8,
}

#[derive(Debug, BorshDeserialize)]
pub struct Depeg {
    pub base_virtual_price: u64,
    pub base_cache_updated: u64,
    pub depeg_type: u8,
}

#[derive(BorshDeserialize)]
pub struct InitializePermissionlessPoolParams {
    pub curve_type: CurveType,
    pub token_a_amount: u64,
    pub token_b_amount: u64,
}

pub fn get_meteoradyn_instruction_type(data: &[u8]) -> Option<MeteoraDynInstructionType> {
    if data.len() < 8 {
        return None;
    }

    let discriminator = &data[0..8];
    if discriminator == INIT_PERMISSIONLESS_POOL_IX_DISCRIMINATOR {
        Some(MeteoraDynInstructionType::InitializePermissionlessPool)
    } else {
        Some(MeteoraDynInstructionType::Unknown)
    }
}

pub fn deserialize_meteoradyn_initialize_permissionless_pool_instruction(
    data: &[u8],
    accounts_indices: &[u8],
    account_keys: &[Pubkey],
    is_signer: &[bool],
    is_writable: &[bool],
) -> Result<JsonValue, String> {
    if data.len() <= 8 {
        return Err("Meteora DYN InitializePermissionlessPool: Data length is too short".to_string());
    }

    if accounts_indices.len() < 24 {
        return Err(format!("Meteora DYN InitializePermissionlessPool: Expected at least 24 accounts, but got {}", accounts_indices.len()));
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
    
    let params = match InitializePermissionlessPoolParams::try_from_slice(&mut remaining_data_ref) {
        Ok(p) => p,
        Err(e) => return Err(format!("Meteora DYN InitializePermissionlessPool: Failed to deserialize parameters: {:?}", e)),
    };

    let curve_type_value = match params.curve_type {
        CurveType::ConstantProduct => "ConstantProduct".to_string(),
        CurveType::Stable { amp, .. } => format!("Stable(amp={})", amp),
    };

    let parsed_data = object! {
        "curve_type" => curve_type_value,
        "token_a_amount" => params.token_a_amount.to_string(),
        "token_b_amount" => params.token_b_amount.to_string()
    };

    Ok(create_standardized_instruction(
        &METEORADYN_PROGRAM_ID,
        "InitializePermissionlessPool",
        "MeteoraDyn",
        data,
        accounts_indices,
        account_keys,
        is_signer,
        is_writable,
        parsed_data
    ))
}