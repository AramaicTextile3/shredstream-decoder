use borsh::{BorshDeserialize, BorshSerialize};
use json::{object, JsonValue};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::pubkey;
use crate::utils::create_standardized_instruction;

pub const PUMP_CREATE_INSTRUCTION_DISCRIMINATOR: [u8; 8] = [0x18, 0x1e, 0xc8, 0x28, 0x05, 0x1c, 0x07, 0x77];
pub const PUMP_BUY_INSTRUCTION_DISCRIMINATOR: [u8; 8] = [0x66, 0x06, 0x3d, 0x12, 0x01, 0xda, 0xeb, 0xea];
// pub const PUMP_TRADE_EVENT_DISCRIMINATOR: [u8; 8] = [0xbd, 0xdb, 0x7f, 0xd3, 0x4e, 0xe6, 0x61, 0xee];

pub const SYSTEM_PROGRAM_ID: Pubkey = pubkey!("11111111111111111111111111111111");
pub const PUMP_MIGRATION_PROGRAM: Pubkey = pubkey!("39azUYFWPz3VHgKCf3VChUwbpURdCHRxjWVowf5jUJjg");
pub const PUMPFUN_PROGRAM_ID: Pubkey = pubkey!("6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P");
pub const PUMPFUN_MINT_AUTHORITY: Pubkey = pubkey!("TSLvdd1pWpHVjahSpsvCXUbgwsL3JAcvokwaKt1eokM");
pub const PUMPFUN_GLOBAL: Pubkey = pubkey!("4wTV1YmiEkRvAtNtsSGPtUrqRYQMe5SKy2uB4Jjaxnjf");
pub const TOKEN_PROGRAM_ID: Pubkey = pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
pub const ASSOCIATED_TOKEN_PROGRAM: Pubkey = pubkey!("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");
pub const RENT_PROGRAM: Pubkey = pubkey!("SysvarRent111111111111111111111111111111111");
pub const EVENT_AUTHORITY: Pubkey = pubkey!("Ce6TQqeHC9p8KetsN6JsjHK7UTZk7nasjjnr7XxXp9F1");
pub const MPL_TOKEN_METADATA: Pubkey = pubkey!("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s");

pub enum PumpfunInstructionType {
    Create,
    Buy,
}

#[derive(BorshDeserialize, BorshSerialize, Debug, Default, Clone)]
pub struct CreateParams {
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub creator: Pubkey,
}

#[derive(BorshDeserialize, BorshSerialize, Debug, Default, Clone)]
pub struct BuyParams {
    amount: u64,
    max_sol_cost: u64,
}

#[derive(BorshDeserialize, BorshSerialize, Debug, Default, Clone)]
pub struct TradeEventParams {
    pub mint: Pubkey,
    pub sol_amount: u64,
    pub token_amount: u64,
    pub is_buy: bool,
    pub user: Pubkey,
    pub timestamp: i64,
    pub virtual_sol_reserves: u64,
    pub virtual_token_reserves: u64,
    pub real_sol_reserves: u64,
    pub real_token_reserves: u64,
}

pub fn get_pumpfun_instruction_type(data: &[u8]) -> Option<PumpfunInstructionType> {
    match data.get(0..8) {
        Some(d) if d == PUMP_CREATE_INSTRUCTION_DISCRIMINATOR => Some(PumpfunInstructionType::Create),
        Some(d) if d == PUMP_BUY_INSTRUCTION_DISCRIMINATOR => Some(PumpfunInstructionType::Buy),
        _ => None,
    }
}

pub fn deserialize_pump_create_instruction(data: &[u8], accounts_indices: &[u8], account_keys: &[Pubkey], is_signer: &[bool], is_writable: &[bool]) -> Result<JsonValue, String> {
    if data.len() < PUMP_CREATE_INSTRUCTION_DISCRIMINATOR.len() {
        return Err("Data length is insufficient for a 'create' instruction.".to_string());
    }

    let remaining_data = &data[8..];
    let mut remaining_data_ref = &remaining_data[..];

    let args = match CreateParams::deserialize(&mut remaining_data_ref) {
        Ok(parsed) => parsed,
        Err(e) => {
            return Err(format!("Failed to deserialize PumpCreateArgs: {:?}", e));
        }
    };

    let parsed_data = object! {
        "name" => args.name,
        "symbol" => args.symbol,
        "uri" => args.uri,
        "creator" => bs58::encode(args.creator.to_bytes()).into_string(),
    };
    
    Ok(create_standardized_instruction(
        &PUMPFUN_PROGRAM_ID,
        "Create",
        "Pumpfun", 
        data,
        accounts_indices,
        account_keys,
        is_signer,
        is_writable,
        parsed_data
    ))
}

pub fn deserialize_pump_buy_instruction(data: &[u8], accounts_indices: &[u8], account_keys: &[Pubkey], is_signer: &[bool], is_writable: &[bool]) -> Result<JsonValue, String> {
    if data.len() < PUMP_BUY_INSTRUCTION_DISCRIMINATOR.len() {
        return Err("Data length is insufficient for a 'buy' instruction.".to_string());
    }
    
    let remaining_data = &data[8..];
    let mut remaining_data_ref = &remaining_data[..];

    let args = match BuyParams::deserialize(&mut remaining_data_ref) {
        Ok(parsed) => parsed,
        Err(e) => {
            return Err(format!("Failed to deserialize PumpBuyParams: {:?}", e));
        }
    };

    let parsed_data = object! {
        "amount" => args.amount.to_string(),
        "max_sol_cost" => args.max_sol_cost.to_string(),
    };
    
    Ok(create_standardized_instruction(
        &PUMPFUN_PROGRAM_ID,
        "Buy",
        "Pumpfun", 
        data,
        accounts_indices,
        account_keys,
        is_signer,
        is_writable,
        parsed_data
    ))
}