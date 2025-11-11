use json::{object, JsonValue};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::pubkey;
use crate::utils::create_standardized_instruction;

pub const METEORA_AMM_V2_PROGRAM_ID: Pubkey = pubkey!("cpamdpZCGKUy5JxQXB4dcpGPiikHawvSWAd6mEn1sGG");

pub const SYSTEM_PROGRAM_ID: Pubkey = pubkey!("11111111111111111111111111111111");
pub const TOKEN_PROGRAM_ID: Pubkey = pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
pub const ASSOCIATED_TOKEN_PROGRAM_ID: Pubkey = pubkey!("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");
pub const METEORA_DAMM_AUTHORITY: Pubkey = pubkey!("HLnpSz9h2S4hiLQ43rnSD9XkcUThA7B8hQMKmDaiTLcC");


pub const CREATE_POOL_1_DISCRIMINATOR: [u8; 8] = [95, 180, 10, 172, 84, 174, 232, 40];
pub const CREATE_POOL_2_DISCRIMINATOR: [u8; 8] = [149, 82, 72, 197, 253, 252, 68, 15];
pub const CREATE_POOL_3_DISCRIMINATOR: [u8; 8] = [20, 161, 241, 24, 189, 221, 180, 2];

// Swap discriminator
pub const SWAP_DISCRIMINATOR: [u8; 8] = [248, 198, 158, 145, 225, 117, 135, 200];

// Add Liquidity discriminators
pub const ADD_LIQUIDITY_1_DISCRIMINATOR: [u8; 8] = [181, 157, 89, 67, 143, 182, 52, 72];
pub const ADD_LIQUIDITY_2_DISCRIMINATOR: [u8; 8] = [95, 180, 10, 172, 84, 174, 232, 40];
pub const ADD_LIQUIDITY_3_DISCRIMINATOR: [u8; 8] = [149, 82, 72, 197, 253, 252, 68, 15];
pub const ADD_LIQUIDITY_4_DISCRIMINATOR: [u8; 8] = [20, 161, 241, 24, 189, 221, 180, 2];

// Remove Liquidity discriminators
pub const REMOVE_LIQUIDITY_1_DISCRIMINATOR: [u8; 8] = [80, 85, 209, 72, 24, 206, 177, 108];
pub const REMOVE_LIQUIDITY_2_DISCRIMINATOR: [u8; 8] = [10, 51, 61, 35, 112, 105, 24, 85];

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum MeteoraAmmV2InstructionType {
    CreatePool1,
    CreatePool2,
    CreatePool3,
    Swap,
    AddLiquidity1,
    AddLiquidity2,
    AddLiquidity3,
    AddLiquidity4,
    RemoveLiquidity1,
    RemoveLiquidity2,
    Unknown,
}

pub fn get_meteora_amm_v2_instruction_type(data: &[u8]) -> Option<MeteoraAmmV2InstructionType> {
    if data.len() < 8 {
        return None;
    }

    let discriminator = &data[0..8];
    match discriminator {
        d if d == CREATE_POOL_1_DISCRIMINATOR => Some(MeteoraAmmV2InstructionType::CreatePool1),
        d if d == CREATE_POOL_2_DISCRIMINATOR => Some(MeteoraAmmV2InstructionType::CreatePool2),
        d if d == CREATE_POOL_3_DISCRIMINATOR => Some(MeteoraAmmV2InstructionType::CreatePool3),
        d if d == SWAP_DISCRIMINATOR => Some(MeteoraAmmV2InstructionType::Swap),
        d if d == ADD_LIQUIDITY_1_DISCRIMINATOR => Some(MeteoraAmmV2InstructionType::AddLiquidity1),
        d if d == ADD_LIQUIDITY_2_DISCRIMINATOR => Some(MeteoraAmmV2InstructionType::AddLiquidity2),
        d if d == ADD_LIQUIDITY_3_DISCRIMINATOR => Some(MeteoraAmmV2InstructionType::AddLiquidity3),
        d if d == ADD_LIQUIDITY_4_DISCRIMINATOR => Some(MeteoraAmmV2InstructionType::AddLiquidity4),
        d if d == REMOVE_LIQUIDITY_1_DISCRIMINATOR => Some(MeteoraAmmV2InstructionType::RemoveLiquidity1),
        d if d == REMOVE_LIQUIDITY_2_DISCRIMINATOR => Some(MeteoraAmmV2InstructionType::RemoveLiquidity2),
        _ => Some(MeteoraAmmV2InstructionType::Unknown),
    }
}

pub fn deserialize_meteora_amm_v2_create_pool_instruction(
    instruction_type: MeteoraAmmV2InstructionType,
    data: &[u8],
    accounts_indices: &[u8],
    account_keys: &[Pubkey],
    is_signer: &[bool],
    is_writable: &[bool]
) -> Result<JsonValue, String> {
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

    let (pool_idx, mint_a_idx, mint_b_idx) = match instruction_type {
        MeteoraAmmV2InstructionType::CreatePool1 => (6, 8, 9),
        MeteoraAmmV2InstructionType::CreatePool2 => (7, 9, 10),
        MeteoraAmmV2InstructionType::CreatePool3 => (5, 7, 8),
        _ => return Err("Invalid create pool instruction type".to_string()),
    };

    let parsed_data = object! {
        "instruction_variant" => format!("{:?}", instruction_type),
        "pool_index" => pool_idx,
        "mint_a_index" => mint_a_idx,
        "mint_b_index" => mint_b_idx,
    };

    Ok(create_standardized_instruction(
        &METEORA_AMM_V2_PROGRAM_ID,
        "CreatePool",
        "MeteoraAmmV2",
        data,
        accounts_indices,
        account_keys,
        is_signer,
        is_writable,
        parsed_data
    ))
}

pub fn deserialize_meteora_amm_v2_swap_instruction(
    data: &[u8],
    accounts_indices: &[u8],
    account_keys: &[Pubkey],
    is_signer: &[bool],
    is_writable: &[bool]
) -> Result<JsonValue, String> {
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

    let parsed_data = object! {
        "instruction_type" => "swap",
    };

    Ok(create_standardized_instruction(
        &METEORA_AMM_V2_PROGRAM_ID,
        "Swap",
        "MeteoraAmmV2",
        data,
        accounts_indices,
        account_keys,
        is_signer,
        is_writable,
        parsed_data
    ))
}

pub fn deserialize_meteora_amm_v2_add_liquidity_instruction(
    instruction_type: MeteoraAmmV2InstructionType,
    data: &[u8],
    accounts_indices: &[u8],
    account_keys: &[Pubkey],
    is_signer: &[bool],
    is_writable: &[bool]
) -> Result<JsonValue, String> {
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

    let (pool_idx, vault_a_idx, vault_b_idx) = match instruction_type {
        MeteoraAmmV2InstructionType::AddLiquidity1 => (0, 4, 5),
        MeteoraAmmV2InstructionType::AddLiquidity2 => (0, 10, 11),
        MeteoraAmmV2InstructionType::AddLiquidity3 => (7, 11, 12),
        MeteoraAmmV2InstructionType::AddLiquidity4 => (5, 9, 10),
        _ => return Err("Invalid add liquidity instruction type".to_string()),
    };

    let parsed_data = object! {
        "instruction_variant" => format!("{:?}", instruction_type),
        "pool_index" => pool_idx,
        "vault_a_index" => vault_a_idx,
        "vault_b_index" => vault_b_idx,
    };

    Ok(create_standardized_instruction(
        &METEORA_AMM_V2_PROGRAM_ID,
        "AddLiquidity",
        "MeteoraAmmV2",
        data,
        accounts_indices,
        account_keys,
        is_signer,
        is_writable,
        parsed_data
    ))
}

pub fn deserialize_meteora_amm_v2_remove_liquidity_instruction(
    instruction_type: MeteoraAmmV2InstructionType,
    data: &[u8],
    accounts_indices: &[u8],
    account_keys: &[Pubkey],
    is_signer: &[bool],
    is_writable: &[bool]
) -> Result<JsonValue, String> {
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

    let parsed_data = object! {
        "instruction_variant" => format!("{:?}", instruction_type),
    };

    Ok(create_standardized_instruction(
        &METEORA_AMM_V2_PROGRAM_ID,
        "RemoveLiquidity",
        "MeteoraAmmV2",
        data,
        accounts_indices,
        account_keys,
        is_signer,
        is_writable,
        parsed_data
    ))
}