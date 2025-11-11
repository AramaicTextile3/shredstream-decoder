use borsh::{BorshDeserialize, BorshSerialize};
use json::{object, JsonValue};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::pubkey;
use crate::utils::create_standardized_instruction;

pub const BOOP_CREATE_TOKEN_INSTRUCTION_DISCRIMINATOR: [u8; 8] = [84, 52, 204, 228, 24, 140, 234, 75];
pub const BOOP_DEPLOY_BONDING_CURVE_INSTRUCTION_DISCRIMINATOR: [u8; 8] = [180, 89, 199, 76, 168, 236, 217, 138];

pub const SYSTEM_PROGRAM_ID: Pubkey = pubkey!("11111111111111111111111111111111");
pub const TOKEN_PROGRAM_ID: Pubkey = pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
pub const ASSOCIATED_TOKEN_PROGRAM: Pubkey = pubkey!("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");
pub const RENT_PROGRAM: Pubkey = pubkey!("SysvarRent111111111111111111111111111111111");
pub const TOKEN_METADATA_PROGRAM: Pubkey = pubkey!("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s");
pub const BOOP_PROGRAM_ID: Pubkey = pubkey!("boop8hVGQGqehUK2iVEMEnMrL5RbjywRzHKBmBE7ry4");

pub enum BoopInstructionType {
    CreateToken,
    DeployBondingCurve,
}

#[derive(BorshDeserialize, BorshSerialize, Debug, Default, Clone)]
pub struct CreateTokenParams {
    pub salt: u64,
    pub name: String,
    pub symbol: String,
    pub uri: String,
}

#[derive(BorshDeserialize, BorshSerialize, Debug, Default, Clone)]
pub struct DeployBondingCurveParams {
    pub creator: Pubkey,
    pub salt: u64,
}

pub fn get_boop_instruction_type(data: &[u8]) -> Option<BoopInstructionType> {
    match data.get(0..8) {
        Some(d) if d == BOOP_CREATE_TOKEN_INSTRUCTION_DISCRIMINATOR => Some(BoopInstructionType::CreateToken),
        Some(d) if d == BOOP_DEPLOY_BONDING_CURVE_INSTRUCTION_DISCRIMINATOR => Some(BoopInstructionType::DeployBondingCurve),
        _ => None,
    }
}

pub fn deserialize_boop_create_token_instruction(data: &[u8], accounts_indices: &[u8], account_keys: &[Pubkey], is_signer: &[bool], is_writable: &[bool]) -> Result<JsonValue, String> {
    if data.len() < BOOP_CREATE_TOKEN_INSTRUCTION_DISCRIMINATOR.len() {
        return Err("Data length is insufficient for a 'create_token' instruction.".to_string());
    }
    let remaining_data = &data[8..];
    let mut remaining_data_ref = &remaining_data[..];

    let args = match CreateTokenParams::deserialize(&mut remaining_data_ref) {
        Ok(parsed) => parsed,
        Err(e) => {
            return Err(format!("Failed to deserialize CreateTokenParams: {:?}", e));
        }
    };

    let parsed_data = object! {
        "salt" => args.salt.to_string(),
        "name" => args.name,
        "symbol" => args.symbol,
        "uri" => args.uri,
    };
    
    Ok(create_standardized_instruction(
        &BOOP_PROGRAM_ID,
        "CreateToken",
        "Boop", 
        data,
        accounts_indices,
        account_keys,
        is_signer,
        is_writable,
        parsed_data
    ))
}

pub fn deserialize_boop_deploy_bonding_curve_instruction(data: &[u8], accounts_indices: &[u8], account_keys: &[Pubkey], is_signer: &[bool], is_writable: &[bool]) -> Result<JsonValue, String> {
    if data.len() < BOOP_DEPLOY_BONDING_CURVE_INSTRUCTION_DISCRIMINATOR.len() {
        return Err("Data length is insufficient for a 'deploy_bonding_curve' instruction.".to_string());
    }

    let remaining_data = &data[8..];
    let mut remaining_data_ref = &remaining_data[..];

    let args = match DeployBondingCurveParams::deserialize(&mut remaining_data_ref) {
        Ok(parsed) => parsed,
        Err(e) => {
            return Err(format!("Failed to deserialize DeployBondingCurveParams: {:?}", e));
        }
    };

    let parsed_data = object! {
        "creator" => bs58::encode(args.creator.to_bytes()).into_string(),
        "salt" => args.salt.to_string(),
    };
    
    Ok(create_standardized_instruction(
        &BOOP_PROGRAM_ID,
        "DeployBondingCurve",
        "Boop", 
        data,
        accounts_indices,
        account_keys,
        is_signer,
        is_writable,
        parsed_data
    ))
}
