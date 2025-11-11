use borsh::{BorshDeserialize, BorshSerialize};
use json::{object, JsonValue};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::pubkey;
use tracing::{error};
use crate::utils::create_standardized_instruction;

// Main trading instruction discriminators from IDL
pub const PUMPAMM_BUY_INSTRUCTION_DISCRIMINATOR: [u8; 8] = [102, 6, 61, 18, 1, 218, 235, 234];
pub const PUMPAMM_SELL_INSTRUCTION_DISCRIMINATOR: [u8; 8] = [51, 230, 133, 164, 1, 127, 131, 173];
pub const PUMPAMM_CREATE_POOL_DISCRIMINATOR: [u8; 8] = [233, 146, 209, 142, 207, 104, 64, 188];
pub const PUMPAMM_DEPOSIT_DISCRIMINATOR: [u8; 8] = [242, 35, 198, 137, 82, 225, 242, 182];
pub const PUMPAMM_WITHDRAW_DISCRIMINATOR: [u8; 8] = [183, 18, 70, 156, 148, 109, 161, 34];

// Administrative instruction discriminators from IDL
pub const PUMPAMM_CREATE_CONFIG_DISCRIMINATOR: [u8; 8] = [201, 207, 243, 114, 75, 111, 47, 189];
pub const PUMPAMM_UPDATE_ADMIN_DISCRIMINATOR: [u8; 8] = [161, 176, 40, 213, 60, 184, 179, 228];
pub const PUMPAMM_UPDATE_FEE_CONFIG_DISCRIMINATOR: [u8; 8] = [104, 184, 103, 242, 88, 151, 107, 20];
pub const PUMPAMM_DISABLE_DISCRIMINATOR: [u8; 8] = [185, 173, 187, 90, 216, 15, 238, 233];
pub const PUMPAMM_SET_COIN_CREATOR_DISCRIMINATOR: [u8; 8] = [210, 149, 128, 45, 188, 58, 78, 175];
pub const PUMPAMM_ADMIN_SET_COIN_CREATOR_DISCRIMINATOR: [u8; 8] = [242, 40, 117, 145, 73, 96, 105, 104];
pub const PUMPAMM_COLLECT_COIN_CREATOR_FEE_DISCRIMINATOR: [u8; 8] = [160, 57, 89, 42, 181, 139, 43, 66];

// Token incentives instruction discriminators from IDL
pub const PUMPAMM_ADMIN_UPDATE_TOKEN_INCENTIVES_DISCRIMINATOR: [u8; 8] = [209, 11, 115, 87, 213, 23, 124, 204];
pub const PUMPAMM_CLAIM_TOKEN_INCENTIVES_DISCRIMINATOR: [u8; 8] = [16, 4, 71, 28, 204, 1, 40, 27];
pub const PUMPAMM_INIT_USER_VOLUME_ACCUMULATOR_DISCRIMINATOR: [u8; 8] = [94, 6, 202, 115, 255, 96, 232, 183];
pub const PUMPAMM_SYNC_USER_VOLUME_ACCUMULATOR_DISCRIMINATOR: [u8; 8] = [86, 31, 192, 87, 163, 87, 79, 238];
pub const PUMPAMM_CLOSE_USER_VOLUME_ACCUMULATOR_DISCRIMINATOR: [u8; 8] = [249, 69, 164, 218, 150, 103, 84, 138];

// Utility instruction discriminators from IDL
pub const PUMPAMM_EXTEND_ACCOUNT_DISCRIMINATOR: [u8; 8] = [234, 102, 194, 203, 150, 72, 62, 229];

pub const PUMPAMM_PROGRAM_ID: Pubkey = pubkey!("pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA");

pub const SYSTEM_PROGRAM_ID: Pubkey = pubkey!("11111111111111111111111111111111");
pub const TOKEN_PROGRAM_ID: Pubkey = pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
pub const TOKEN_2022_PROGRAM_ID: Pubkey = pubkey!("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");
pub const ASSOCIATED_TOKEN_PROGRAM_ID: Pubkey = pubkey!("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");
pub const PROTOCOL_FEE_RECIPIENT_TOKEN_ACCOUNT: Pubkey = pubkey!("BWXT6RUhit9FfJQM3pBmqeFLPYmuxgmyhMGC5sGr8RbA");
pub const PUMPAMM_EVENT_AUTHORITY: Pubkey = pubkey!("GS4CU59F31iL7aR2Q8zVS8DRrcRnXX1yjQ66TqNVQnaR");
pub const GLOBAL_CONFIG: Pubkey = pubkey!("ADyA8hdefvWN2dbGGWFotbzWxrAvLW83WG6QCVXvJKqw");

pub enum PumpAmmInstructionType {
    // Main trading instructions
    Buy,
    Sell,
    CreatePool,
    Deposit,
    Withdraw,
    
    Unknown,
}

#[derive(BorshDeserialize, BorshSerialize, Debug, Default, Clone)]
pub struct BuyParams {
    pub base_amount_out: u64,
    pub max_quote_amount_in: u64,
}

#[derive(BorshDeserialize, BorshSerialize, Debug, Default, Clone)]
pub struct SellParams {
    pub base_amount_in: u64,
    pub min_quote_amount_out: u64,
}

#[derive(BorshDeserialize, BorshSerialize, Debug, Default, Clone)]
pub struct CreatePoolParams {
    pub index: u16,
    pub base_amount_in: u64,
    pub quote_amount_in: u64,
    pub coin_creator: Pubkey,
}

#[derive(BorshDeserialize, BorshSerialize, Debug, Default, Clone)]
pub struct DepositParams {
    pub lp_token_amount_out: u64,
    pub max_base_amount_in: u64,
    pub max_quote_amount_in: u64,
}

#[derive(BorshDeserialize, BorshSerialize, Debug, Default, Clone)]
pub struct WithdrawParams {
    pub lp_token_amount_in: u64,
    pub min_base_amount_out: u64,
    pub min_quote_amount_out: u64,
}

pub fn get_pumpamm_instruction_type(data: &[u8]) -> Option<PumpAmmInstructionType> {
    if data.len() < 8 {
        return None;
    }
    
    let discriminator = &data[0..8];
    
    match discriminator {
        // Main trading instructions
        d if d == PUMPAMM_BUY_INSTRUCTION_DISCRIMINATOR => Some(PumpAmmInstructionType::Buy),
        d if d == PUMPAMM_SELL_INSTRUCTION_DISCRIMINATOR => Some(PumpAmmInstructionType::Sell),
        d if d == PUMPAMM_CREATE_POOL_DISCRIMINATOR => Some(PumpAmmInstructionType::CreatePool),
        d if d == PUMPAMM_DEPOSIT_DISCRIMINATOR => Some(PumpAmmInstructionType::Deposit),
        d if d == PUMPAMM_WITHDRAW_DISCRIMINATOR => Some(PumpAmmInstructionType::Withdraw),
        
        _ => Some(PumpAmmInstructionType::Unknown),
    }
}

pub fn deserialize_pumpamm_instruction(data: &[u8], accounts_indices: &[u8], account_keys: &[Pubkey], is_signer: &[bool], is_writable: &[bool]) -> Result<JsonValue, String> {
    if data.len() < 8 {
        return Err("Data is too short for a PumpAMM instruction.".to_string());
    }

    let instruction_type = get_pumpamm_instruction_type(data);
    
    match instruction_type {
        // Main trading instructions - fully implemented
        Some(PumpAmmInstructionType::Buy) => deserialize_pumpamm_buy_instruction(data, accounts_indices, account_keys, is_signer, is_writable),
        Some(PumpAmmInstructionType::Sell) => deserialize_pumpamm_sell_instruction(data, accounts_indices, account_keys, is_signer, is_writable),
        Some(PumpAmmInstructionType::CreatePool) => deserialize_pumpamm_create_pool_instruction(data, accounts_indices, account_keys, is_signer, is_writable),
        Some(PumpAmmInstructionType::Deposit) => deserialize_pumpamm_deposit_instruction(data, accounts_indices, account_keys, is_signer, is_writable),
        Some(PumpAmmInstructionType::Withdraw) => deserialize_pumpamm_withdraw_instruction(data, accounts_indices, account_keys, is_signer, is_writable),
        
        Some(PumpAmmInstructionType::Unknown) => Err("Unknown PumpAMM instruction type.".to_string()),
        None => Err("Could not identify the PumpAMM instruction type.".to_string()),
    }
}

pub fn deserialize_pumpamm_buy_instruction(data: &[u8], accounts_indices: &[u8], account_keys: &[Pubkey], is_signer: &[bool], is_writable: &[bool]) -> Result<JsonValue, String> {
    if data.len() < PUMPAMM_BUY_INSTRUCTION_DISCRIMINATOR.len() {
        return Err("Data is insufficient for a 'buy' instruction.".to_string());
    }

    if accounts_indices.len() < 17 {
        return Err(format!("Insufficient number of accounts for 'buy' instruction: {} (minimum required: 17)", accounts_indices.len()));
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
    let params = match BuyParams::deserialize(&mut &remaining_data[..]) {
        Ok(parsed) => parsed,
        Err(e) => {
            return Err(format!("Error deserializing Buy parameters: {:?}", e));
        }
    };

    let parsed_data = object! {
        "base_amount_out" => params.base_amount_out.to_string(),
        "max_quote_amount_in" => params.max_quote_amount_in.to_string()
    };
    
    Ok(create_standardized_instruction(
        &PUMPAMM_PROGRAM_ID,
        "Buy",
        "PumpAMM", 
        data,
        accounts_indices,
        account_keys,
        is_signer,
        is_writable,
        parsed_data
    ))
}

pub fn deserialize_pumpamm_sell_instruction(data: &[u8], accounts_indices: &[u8], account_keys: &[Pubkey], is_signer: &[bool], is_writable: &[bool]) -> Result<JsonValue, String> {
    if data.len() < PUMPAMM_SELL_INSTRUCTION_DISCRIMINATOR.len() {
        return Err("Data is too short for a PumpAMM Sell instruction.".to_string());
    }

    if accounts_indices.len() < 17 {
        return Err(format!("Insufficient number of accounts for 'sell' instruction: {} (minimum required: 17)", accounts_indices.len()));
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
    let params = match SellParams::deserialize(&mut &remaining_data[..]) {
        Ok(parsed) => parsed,
        Err(e) => {
            return Err(format!("Error deserializing PSell parameters: {:?}", e));
        }
    };

    let parsed_data = object! {
        "base_amount_in" => params.base_amount_in.to_string(),
        "min_quote_amount_out" => params.min_quote_amount_out.to_string(),
    };
    
    Ok(create_standardized_instruction(
        &PUMPAMM_PROGRAM_ID,
        "Sell",
        "PumpAMM", 
        data,
        accounts_indices,
        account_keys,
        is_signer,
        is_writable,
        parsed_data
    ))
}

pub fn deserialize_pumpamm_create_pool_instruction(data: &[u8], accounts_indices: &[u8], account_keys: &[Pubkey], is_signer: &[bool], is_writable: &[bool]) -> Result<JsonValue, String> {
    if data.len() < PUMPAMM_CREATE_POOL_DISCRIMINATOR.len() {
        return Err("Data is too short for a PumpAMM CreatePool instruction.".to_string());
    }

    let discriminator = &data[0..8];
    if discriminator != PUMPAMM_CREATE_POOL_DISCRIMINATOR {
        return Err("Not a PumpAMM CreatePool instruction.".to_string());
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

    let args = match CreatePoolParams::deserialize(&mut remaining_data_ref) {
        Ok(parsed) => parsed,
        Err(e) => {
            error!("Failed to deserialize PumpAMM CreatePoolParams: {:?}", e);
            return Err(format!("Failed to deserialize PumpAMM CreatePoolParams: {:?}", e));
        }
    };

    let parsed_data = object! {
        "index" => args.index.to_string(),
        "baseAmountIn" => args.base_amount_in.to_string(),
        "quoteAmountIn" => args.quote_amount_in.to_string(),
        "coinCreator" => bs58::encode(args.coin_creator.to_bytes()).into_string()
    };
    
    Ok(create_standardized_instruction(
        &PUMPAMM_PROGRAM_ID,
        "CreatePool",
        "PumpAMM", 
        data,
        accounts_indices,
        account_keys,
        is_signer,
        is_writable,
        parsed_data
    ))
}

pub fn deserialize_pumpamm_deposit_instruction(data: &[u8], accounts_indices: &[u8], account_keys: &[Pubkey], is_signer: &[bool], is_writable: &[bool]) -> Result<JsonValue, String> {
    if data.len() < PUMPAMM_DEPOSIT_DISCRIMINATOR.len() {
        return Err("Data is insufficient for a 'deposit' instruction.".to_string());
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
    let params = match DepositParams::deserialize(&mut &remaining_data[..]) {
        Ok(parsed) => parsed,
        Err(e) => {
            return Err(format!("Error deserializing Deposit parameters: {:?}", e));
        }
    };

    let parsed_data = object! {
        "lpTokenAmountOut" => params.lp_token_amount_out.to_string(),
        "maxBaseAmountIn" => params.max_base_amount_in.to_string(),
        "maxQuoteAmountIn" => params.max_quote_amount_in.to_string()
    };
    
    Ok(create_standardized_instruction(
        &PUMPAMM_PROGRAM_ID,
        "Deposit",
        "PumpAMM", 
        data,
        accounts_indices,
        account_keys,
        is_signer,
        is_writable,
        parsed_data
    ))
}

pub fn deserialize_pumpamm_withdraw_instruction(data: &[u8], accounts_indices: &[u8], account_keys: &[Pubkey], is_signer: &[bool], is_writable: &[bool]) -> Result<JsonValue, String> {
    if data.len() < PUMPAMM_WITHDRAW_DISCRIMINATOR.len() {
        return Err("Data is insufficient for a 'withdraw' instruction.".to_string());
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
    let params = match WithdrawParams::deserialize(&mut &remaining_data[..]) {
        Ok(parsed) => parsed,
        Err(e) => {
            return Err(format!("Error deserializing Withdraw parameters: {:?}", e));
        }
    };

    let parsed_data = object! {
        "lpTokenAmountIn" => params.lp_token_amount_in.to_string(),
        "minBaseAmountOut" => params.min_base_amount_out.to_string(),
        "minQuoteAmountOut" => params.min_quote_amount_out.to_string()
    };
    
    Ok(create_standardized_instruction(
        &PUMPAMM_PROGRAM_ID,
        "Withdraw",
        "PumpAMM", 
        data,
        accounts_indices,
        account_keys,
        is_signer,
        is_writable,
        parsed_data
    ))
}

pub fn deserialize_pumpamm_generic_instruction(instruction_name: &str, data: &[u8], accounts_indices: &[u8], account_keys: &[Pubkey], is_signer: &[bool], is_writable: &[bool]) -> Result<JsonValue, String> {
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
        "discriminator" => format!("{:?}", &data[0..8.min(data.len())]),
        "note" => "Generic instruction parsing - detailed parameter parsing not implemented"
    };
    
    Ok(create_standardized_instruction(
        &PUMPAMM_PROGRAM_ID,
        instruction_name,
        "PumpAMM", 
        data,
        accounts_indices,
        account_keys,
        is_signer,
        is_writable,
        parsed_data
    ))
}