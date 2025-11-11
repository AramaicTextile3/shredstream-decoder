use borsh::{BorshDeserialize, BorshSerialize};
use json::{object, JsonValue};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::pubkey;
use crate::utils::create_standardized_instruction;


pub const ORCA_SWAP_INSTRUCTION_DISCRIMINATOR: [u8; 8] = [248, 198, 158, 145, 225, 117, 135, 200];
pub const ORCA_INCREASE_LIQUIDITY_INSTRUCTION_DISCRIMINATOR: [u8; 8] = [46, 156, 243, 118, 13, 205, 251, 178];
pub const ORCA_DECREASE_LIQUIDITY_INSTRUCTION_DISCRIMINATOR: [u8; 8] = [58, 127, 188, 62, 79, 82, 196, 96];
pub const ORCA_INITIALIZE_POOL_INSTRUCTION_DISCRIMINATOR: [u8; 8] = [95, 180, 10, 172, 84, 174, 232, 40];
pub const ORCA_OPEN_POSITION_INSTRUCTION_DISCRIMINATOR: [u8; 8] = [135, 128, 47, 77, 15, 152, 240, 49];
pub const ORCA_CLOSE_POSITION_INSTRUCTION_DISCRIMINATOR: [u8; 8] = [123, 134, 81, 0, 49, 68, 98, 98];
pub const ORCA_SWAP_V2_INSTRUCTION_DISCRIMINATOR: [u8; 8] = [43, 4, 237, 11, 26, 201, 30, 98];
pub const ORCA_TWO_HOP_SWAP_INSTRUCTION_DISCRIMINATOR: [u8; 8] = [235, 47, 68, 120, 187, 155, 176, 203];

pub const ORCA_WHIRLPOOL_PROGRAM_ID: Pubkey = pubkey!("whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc");
pub const SYSTEM_PROGRAM_ID: Pubkey = pubkey!("11111111111111111111111111111111");
pub const TOKEN_PROGRAM_ID: Pubkey = pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
pub const TOKEN_2022_PROGRAM_ID: Pubkey = pubkey!("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");
pub const ASSOCIATED_TOKEN_PROGRAM_ID: Pubkey = pubkey!("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");

pub enum OrcaInstructionType {
    Swap,
    IncreaseLiquidity,
    DecreaseLiquidity,
    InitializePool,
    OpenPosition,
    ClosePosition,
    SwapV2,
    TwoHopSwap,
}

#[derive(BorshDeserialize, BorshSerialize, Debug, Default, Clone)]
pub struct SwapParams {
    pub amount: u64,
    pub other_amount_threshold: u64,
    pub sqrt_price_limit: u128,
    pub amount_specified_is_input: bool,
    pub a_to_b: bool,
}

#[derive(BorshDeserialize, BorshSerialize, Debug, Default, Clone)]
pub struct SwapV2Params {
    pub amount: u64,
    pub other_amount_threshold: u64,
    pub sqrt_price_limit: u128,
    pub amount_specified_is_input: bool,
    pub a_to_b: bool,
    pub remaining_accounts_info: Option<RemainingAccountsInfo>,
}

#[derive(BorshDeserialize, BorshSerialize, Debug, Default, Clone)]
pub struct IncreaseLiquidityParams {
    pub liquidity_amount: u128,
    pub token_max_a: u64,
    pub token_max_b: u64,
}

#[derive(BorshDeserialize, BorshSerialize, Debug, Default, Clone)]
pub struct DecreaseLiquidityParams {
    pub liquidity_amount: u128,
    pub token_min_a: u64,
    pub token_min_b: u64,
}

#[derive(BorshDeserialize, BorshSerialize, Debug, Default, Clone)]
pub struct InitializePoolParams {
    pub bumps: WhirlpoolBumps,
    pub tick_spacing: u16,
    pub initial_sqrt_price: u128,
}

#[derive(BorshDeserialize, BorshSerialize, Debug, Default, Clone)]
pub struct OpenPositionParams {
    pub bumps: OpenPositionBumps,
    pub tick_lower_index: i32,
    pub tick_upper_index: i32,
}

#[derive(BorshDeserialize, BorshSerialize, Debug, Default, Clone)]
pub struct TwoHopSwapParams {
    pub amount: u64,
    pub other_amount_threshold: u64,
    pub amount_specified_is_input: bool,
    pub a_to_b_one: bool,
    pub a_to_b_two: bool,
    pub sqrt_price_limit_one: u128,
    pub sqrt_price_limit_two: u128,
}

#[derive(BorshDeserialize, BorshSerialize, Debug, Default, Clone)]
pub struct WhirlpoolBumps {
    pub whirlpool_bump: u8,
}

#[derive(BorshDeserialize, BorshSerialize, Debug, Default, Clone)]
pub struct OpenPositionBumps {
    pub position_bump: u8,
}

#[derive(BorshDeserialize, BorshSerialize, Debug, Default, Clone)]
pub struct RemainingAccountsInfo {
    pub slices: Vec<RemainingAccountsSlice>,
}

#[derive(BorshDeserialize, BorshSerialize, Debug, Default, Clone)]
pub struct RemainingAccountsSlice {
    pub accounts_type: AccountsType,
    pub length: u8,
}

#[derive(BorshDeserialize, BorshSerialize, Debug, Clone)]
pub enum AccountsType {
    TransferHookA,
    TransferHookB,
    TransferHookReward,
    TransferHookInput,
    TransferHookIntermediate,
    TransferHookOutput,
    SupplementalTickArrays,
    SupplementalTickArraysOne,
    SupplementalTickArraysTwo,
}

impl Default for AccountsType {
    fn default() -> Self {
        AccountsType::TransferHookA
    }
}

pub fn get_orca_instruction_type(data: &[u8]) -> Option<OrcaInstructionType> {
    match data.get(0..8) {
        Some(d) if d == ORCA_SWAP_INSTRUCTION_DISCRIMINATOR => Some(OrcaInstructionType::Swap),
        Some(d) if d == ORCA_SWAP_V2_INSTRUCTION_DISCRIMINATOR => Some(OrcaInstructionType::SwapV2),
        Some(d) if d == ORCA_INCREASE_LIQUIDITY_INSTRUCTION_DISCRIMINATOR => Some(OrcaInstructionType::IncreaseLiquidity),
        Some(d) if d == ORCA_DECREASE_LIQUIDITY_INSTRUCTION_DISCRIMINATOR => Some(OrcaInstructionType::DecreaseLiquidity),
        Some(d) if d == ORCA_INITIALIZE_POOL_INSTRUCTION_DISCRIMINATOR => Some(OrcaInstructionType::InitializePool),
        Some(d) if d == ORCA_OPEN_POSITION_INSTRUCTION_DISCRIMINATOR => Some(OrcaInstructionType::OpenPosition),
        Some(d) if d == ORCA_CLOSE_POSITION_INSTRUCTION_DISCRIMINATOR => Some(OrcaInstructionType::ClosePosition),
        Some(d) if d == ORCA_TWO_HOP_SWAP_INSTRUCTION_DISCRIMINATOR => Some(OrcaInstructionType::TwoHopSwap),
        _ => None,
    }
}

pub fn deserialize_orca_swap_instruction(data: &[u8], accounts_indices: &[u8], account_keys: &[Pubkey], is_signer: &[bool], is_writable: &[bool]) -> Result<JsonValue, String> {
    if data.len() < ORCA_SWAP_INSTRUCTION_DISCRIMINATOR.len() {
        return Err("Data length is insufficient for a 'swap' instruction.".to_string());
    }

    let safely_get_account = |idx: usize| -> String {
        if idx >= accounts_indices.len() {
            return "Unknown".to_string();
        }
        
        let account_idx = accounts_indices[idx] as usize;
        if account_idx >= account_keys.len() {
            return "Unknown".to_string();
        }
        
        account_keys[account_idx].to_string()
    };

    let params = SwapParams::try_from_slice(&data[8..]).map_err(|e| format!("Failed to deserialize swap params: {}", e))?;

    let parsed_data = object! {
        "amount" => params.amount.to_string(),
        "otherAmountThreshold" => params.other_amount_threshold.to_string(),
        "sqrtPriceLimit" => params.sqrt_price_limit.to_string(),
        "amountSpecifiedIsInput" => params.amount_specified_is_input,
        "aToB" => params.a_to_b,
    };

    Ok(create_standardized_instruction(
        &ORCA_WHIRLPOOL_PROGRAM_ID,
        "Swap",
        "Orca",
        data,
        accounts_indices,
        account_keys,
        is_signer,
        is_writable,
        parsed_data
    ))
}

pub fn deserialize_orca_swap_v2_instruction(data: &[u8], accounts_indices: &[u8], account_keys: &[Pubkey], is_signer: &[bool], is_writable: &[bool]) -> Result<JsonValue, String> {
    if data.len() < ORCA_SWAP_V2_INSTRUCTION_DISCRIMINATOR.len() {
        return Err("Data length is insufficient for a 'swapV2' instruction.".to_string());
    }

    let safely_get_account = |idx: usize| -> String {
        if idx >= accounts_indices.len() {
            return "Unknown".to_string();
        }
        
        let account_idx = accounts_indices[idx] as usize;
        if account_idx >= account_keys.len() {
            return "Unknown".to_string();
        }
        
        account_keys[account_idx].to_string()
    };

    let params = SwapV2Params::try_from_slice(&data[8..]).map_err(|e| format!("Failed to deserialize swapV2 params: {}", e))?;

    let parsed_data = object! {
        "amount" => params.amount.to_string(),
        "otherAmountThreshold" => params.other_amount_threshold.to_string(),
        "sqrtPriceLimit" => params.sqrt_price_limit.to_string(),
        "amountSpecifiedIsInput" => params.amount_specified_is_input,
        "aToB" => params.a_to_b,
    };

    Ok(create_standardized_instruction(
        &ORCA_WHIRLPOOL_PROGRAM_ID,
        "SwapV2",
        "Orca",
        data,
        accounts_indices,
        account_keys,
        is_signer,
        is_writable,
        parsed_data
    ))
}

pub fn deserialize_orca_increase_liquidity_instruction(data: &[u8], accounts_indices: &[u8], account_keys: &[Pubkey], is_signer: &[bool], is_writable: &[bool]) -> Result<JsonValue, String> {
    if data.len() < ORCA_INCREASE_LIQUIDITY_INSTRUCTION_DISCRIMINATOR.len() {
        return Err("Data length is insufficient for an 'increaseLiquidity' instruction.".to_string());
    }

    let safely_get_account = |idx: usize| -> String {
        if idx >= accounts_indices.len() {
            return "Unknown".to_string();
        }
        
        let account_idx = accounts_indices[idx] as usize;
        if account_idx >= account_keys.len() {
            return "Unknown".to_string();
        }
        
        account_keys[account_idx].to_string()
    };

    let params = IncreaseLiquidityParams::try_from_slice(&data[8..]).map_err(|e| format!("Failed to deserialize increaseLiquidity params: {}", e))?;

    let parsed_data = object! {
        "liquidityAmount" => params.liquidity_amount.to_string(),
        "tokenMaxA" => params.token_max_a.to_string(),
        "tokenMaxB" => params.token_max_b.to_string(),
    };

    Ok(create_standardized_instruction(
        &ORCA_WHIRLPOOL_PROGRAM_ID,
        "IncreaseLiquidity",
        "Orca",
        data,
        accounts_indices,
        account_keys,
        is_signer,
        is_writable,
        parsed_data
    ))
}

pub fn deserialize_orca_decrease_liquidity_instruction(data: &[u8], accounts_indices: &[u8], account_keys: &[Pubkey], is_signer: &[bool], is_writable: &[bool]) -> Result<JsonValue, String> {
    if data.len() < ORCA_DECREASE_LIQUIDITY_INSTRUCTION_DISCRIMINATOR.len() {
        return Err("Data length is insufficient for a 'decreaseLiquidity' instruction.".to_string());
    }

    let safely_get_account = |idx: usize| -> String {
        if idx >= accounts_indices.len() {
            return "Unknown".to_string();
        }
        
        let account_idx = accounts_indices[idx] as usize;
        if account_idx >= account_keys.len() {
            return "Unknown".to_string();
        }
        
        account_keys[account_idx].to_string()
    };

    let params = DecreaseLiquidityParams::try_from_slice(&data[8..]).map_err(|e| format!("Failed to deserialize decreaseLiquidity params: {}", e))?;

    let parsed_data = object! {
        "liquidityAmount" => params.liquidity_amount.to_string(),
        "tokenMinA" => params.token_min_a.to_string(),
        "tokenMinB" => params.token_min_b.to_string(),
    };

    Ok(create_standardized_instruction(
        &ORCA_WHIRLPOOL_PROGRAM_ID,
        "DecreaseLiquidity",
        "Orca",
        data,
        accounts_indices,
        account_keys,
        is_signer,
        is_writable,
        parsed_data
    ))
}

pub fn deserialize_orca_initialize_pool_instruction(data: &[u8], accounts_indices: &[u8], account_keys: &[Pubkey], is_signer: &[bool], is_writable: &[bool]) -> Result<JsonValue, String> {
    if data.len() < ORCA_INITIALIZE_POOL_INSTRUCTION_DISCRIMINATOR.len() {
        return Err("Data length is insufficient for an 'initializePool' instruction.".to_string());
    }

    let safely_get_account = |idx: usize| -> String {
        if idx >= accounts_indices.len() {
            return "Unknown".to_string();
        }
        
        let account_idx = accounts_indices[idx] as usize;
        if account_idx >= account_keys.len() {
            return "Unknown".to_string();
        }
        
        account_keys[account_idx].to_string()
    };

    let params = InitializePoolParams::try_from_slice(&data[8..]).map_err(|e| format!("Failed to deserialize initializePool params: {}", e))?;

    let parsed_data = object! {
        "tickSpacing" => params.tick_spacing,
        "initialSqrtPrice" => params.initial_sqrt_price.to_string(),
    };

    Ok(create_standardized_instruction(
        &ORCA_WHIRLPOOL_PROGRAM_ID,
        "InitializePool",
        "Orca",
        data,
        accounts_indices,
        account_keys,
        is_signer,
        is_writable,
        parsed_data
    ))
}

pub fn deserialize_orca_open_position_instruction(data: &[u8], accounts_indices: &[u8], account_keys: &[Pubkey], is_signer: &[bool], is_writable: &[bool]) -> Result<JsonValue, String> {
    if data.len() < ORCA_OPEN_POSITION_INSTRUCTION_DISCRIMINATOR.len() {
        return Err("Data length is insufficient for an 'openPosition' instruction.".to_string());
    }

    let safely_get_account = |idx: usize| -> String {
        if idx >= accounts_indices.len() {
            return "Unknown".to_string();
        }
        
        let account_idx = accounts_indices[idx] as usize;
        if account_idx >= account_keys.len() {
            return "Unknown".to_string();
        }
        
        account_keys[account_idx].to_string()
    };

    let params = OpenPositionParams::try_from_slice(&data[8..]).map_err(|e| format!("Failed to deserialize openPosition params: {}", e))?;

    let parsed_data = object! {
        "tickLowerIndex" => params.tick_lower_index,
        "tickUpperIndex" => params.tick_upper_index,
    };

    Ok(create_standardized_instruction(
        &ORCA_WHIRLPOOL_PROGRAM_ID,
        "OpenPosition",
        "Orca",
        data,
        accounts_indices,
        account_keys,
        is_signer,
        is_writable,
        parsed_data
    ))
}

pub fn deserialize_orca_close_position_instruction(data: &[u8], accounts_indices: &[u8], account_keys: &[Pubkey], is_signer: &[bool], is_writable: &[bool]) -> Result<JsonValue, String> {
    if data.len() < ORCA_CLOSE_POSITION_INSTRUCTION_DISCRIMINATOR.len() {
        return Err("Data length is insufficient for a 'closePosition' instruction.".to_string());
    }

    let safely_get_account = |idx: usize| -> String {
        if idx >= accounts_indices.len() {
            return "Unknown".to_string();
        }
        
        let account_idx = accounts_indices[idx] as usize;
        if account_idx >= account_keys.len() {
            return "Unknown".to_string();
        }
        
        account_keys[account_idx].to_string()
    };

    let parsed_data = object! {};

    Ok(create_standardized_instruction(
        &ORCA_WHIRLPOOL_PROGRAM_ID,
        "ClosePosition",
        "Orca",
        data,
        accounts_indices,
        account_keys,
        is_signer,
        is_writable,
        parsed_data
    ))
}

pub fn deserialize_orca_two_hop_swap_instruction(data: &[u8], accounts_indices: &[u8], account_keys: &[Pubkey], is_signer: &[bool], is_writable: &[bool]) -> Result<JsonValue, String> {
    if data.len() < ORCA_TWO_HOP_SWAP_INSTRUCTION_DISCRIMINATOR.len() {
        return Err("Data length is insufficient for a 'twoHopSwap' instruction.".to_string());
    }

    let safely_get_account = |idx: usize| -> String {
        if idx >= accounts_indices.len() {
            return "Unknown".to_string();
        }
        
        let account_idx = accounts_indices[idx] as usize;
        if account_idx >= account_keys.len() {
            return "Unknown".to_string();
        }
        
        account_keys[account_idx].to_string()
    };

    let params = TwoHopSwapParams::try_from_slice(&data[8..]).map_err(|e| format!("Failed to deserialize twoHopSwap params: {}", e))?;

    let parsed_data = object! {
        "amount" => params.amount.to_string(),
        "otherAmountThreshold" => params.other_amount_threshold.to_string(),
        "amountSpecifiedIsInput" => params.amount_specified_is_input,
        "aToBOne" => params.a_to_b_one,
        "aToBTwo" => params.a_to_b_two,
        "sqrtPriceLimitOne" => params.sqrt_price_limit_one.to_string(),
        "sqrtPriceLimitTwo" => params.sqrt_price_limit_two.to_string(),
    };

    Ok(create_standardized_instruction(
        &ORCA_WHIRLPOOL_PROGRAM_ID,
        "TwoHopSwap",
        "Orca",
        data,
        accounts_indices,
        account_keys,
        is_signer,
        is_writable,
        parsed_data
    ))
}