use borsh::{BorshDeserialize, BorshSerialize};
use json::{object, JsonValue};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::pubkey;
use crate::utils::create_standardized_instruction;

pub const MOONIT_TOKEN_MINT_INSTRUCTION_DISCRIMINATOR: [u8; 8] = [3, 44, 164, 184, 123, 13, 245, 179];

pub const MOONIT_PROGRAM_ID: Pubkey = pubkey!("MoonCVVNZFSYkqNXP6bxHLPL6QQJiMagDL3qcqUQTrG");
pub const SYSTEM_PROGRAM_ID: Pubkey = pubkey!("11111111111111111111111111111111");
pub const TOKEN_PROGRAM_ID: Pubkey = pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
pub const ASSOCIATED_TOKEN_PROGRAM: Pubkey = pubkey!("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");
pub const MPL_TOKEN_METADATA: Pubkey = pubkey!("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s");

pub enum MoonitInstructionType {
    TokenMint,
}

#[derive(BorshDeserialize, BorshSerialize, Debug, Default, Clone)]
pub struct TokenMintParams {
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub decimals: u8,
    pub collateral_currency: u8,
    pub amount: u64,
    pub curve_type: u8,
    pub migration_target: u8,
}

pub fn get_moonit_instruction_type(data: &[u8]) -> Option<MoonitInstructionType> {
    match data.get(0..8) {
        Some(d) if d == MOONIT_TOKEN_MINT_INSTRUCTION_DISCRIMINATOR => Some(MoonitInstructionType::TokenMint),
        _ => None,
    }
}

pub fn deserialize_moonit_token_mint_instruction(data: &[u8], accounts_indices: &[u8], account_keys: &[Pubkey], is_signer: &[bool], is_writable: &[bool]) -> Result<JsonValue, String> {
    if data.len() < MOONIT_TOKEN_MINT_INSTRUCTION_DISCRIMINATOR.len() {
        return Err("Data length is insufficient for a 'tokenMint' instruction.".to_string());
    }

    let remaining_data = &data[8..];
    let mut remaining_data_ref = &remaining_data[..];

    let args = match TokenMintParams::deserialize(&mut remaining_data_ref) {
        Ok(parsed) => parsed,
        Err(e) => {
            return Err(format!("Failed to deserialize TokenMintParams: {:?}", e));
        }
    };

    let curve_type = match args.curve_type {
        0 => "LinearV1",
        1 => "ConstantProductV1",
        _ => "Unknown",
    };

    let migration_target = match args.migration_target {
        0 => "Raydium",
        1 => "Meteora",
        _ => "Unknown",
    };

    let collateral_currency = match args.collateral_currency {
        0 => "Sol",
        _ => "Unknown",
    };

    let parsed_data = object! {
        "name" => args.name,
        "symbol" => args.symbol,
        "uri" => args.uri,
        "decimals" => args.decimals.to_string(),
        "collateral_currency" => collateral_currency,
        "amount" => args.amount.to_string(),
        "curve_type" => curve_type,
        "migration_target" => migration_target,
    };
    
    Ok(create_standardized_instruction(
        &MOONIT_PROGRAM_ID,
        "TokenMint",
        "Moonit", 
        data,
        accounts_indices,
        account_keys,
        is_signer,
        is_writable,
        parsed_data
    ))
}
