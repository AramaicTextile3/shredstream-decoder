use borsh::{BorshDeserialize, BorshSerialize};
use json::{object, JsonValue};
use solana_sdk::pubkey;
use solana_sdk::pubkey::Pubkey;
use crate::utils::create_standardized_instruction;

pub const RAYDIUM_CPMM_PROGRAM: Pubkey = pubkey!("CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C");

pub const SYSTEM_PROGRAM_ID: Pubkey = pubkey!("11111111111111111111111111111111");
pub const TOKEN_PROGRAM_ID: Pubkey = pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
pub const TOKEN_PROGRAM_2022: Pubkey = pubkey!("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");
pub const ASSOCIATED_TOKEN_PROGRAM: Pubkey = pubkey!("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");
pub const RENT_PROGRAM: Pubkey = pubkey!("SysvarRent111111111111111111111111111111111");

const INITIALIZE_DISCRIMINATOR: [u8; 8] = [175, 175, 109, 31, 13, 152, 155, 237];

#[derive(BorshDeserialize, BorshSerialize, Debug, Default, Clone)]
pub struct InitializeParams {
    pub init_amount_0: u64,
    pub init_amount_1: u64,
    pub open_time: u64,
}

pub enum RaydiumCpmmInstructionType {
    Initialize,
}

pub fn get_raydium_cpmm_instruction_type(data: &[u8]) -> Option<RaydiumCpmmInstructionType> {
    if data.len() < 8 {
        return None;
    }
    
    let discriminator = &data[0..8];
    
    if discriminator == INITIALIZE_DISCRIMINATOR {
        Some(RaydiumCpmmInstructionType::Initialize)
    } else {
        None
    }
}

pub fn deserialize_raydium_cpmm_initialize_instruction(
    data: &[u8],
    accounts_indices: &[u8],
    account_keys: &[Pubkey],
    is_signer: &[bool],
    is_writable: &[bool],
) -> Result<JsonValue, String> {
    let mut data_ref = &data[8..];

    let args = match InitializeParams::deserialize(&mut data_ref) {
        Ok(parsed) => parsed,
        Err(e) => {
            return Err(format!("InitializeParams deserialization failure: {:?}", e));
        }
    };
    
    if accounts_indices.len() < 20 {
        return Err(format!(
            "Insufficient number of accounts to initialize: {} (minimum 20 required)",
            accounts_indices.len()
        ));
    }

    let parsed_data = object! {
        "init_amount_0" => args.init_amount_0.to_string(),
        "init_amount_1" => args.init_amount_1.to_string(),
        "open_time" => args.open_time.to_string(),
    };
    
    Ok(create_standardized_instruction(
        &RAYDIUM_CPMM_PROGRAM,
        "Initialize",
        "RaydiumCPMM", 
        data,
        accounts_indices,
        account_keys,
        is_signer,
        is_writable,
        parsed_data
    ))
}
