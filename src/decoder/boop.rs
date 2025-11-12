use json::{object, JsonValue};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::pubkey;
use tracing::{error, warn};

use crate::decoder::decoder::InstructionDecoder;
use crate::decoder::byte_parser::{parse_string, parse_u64, parse_pubkey};
use crate::utils::create_standardized_instruction;

pub const BOOP_CREATE_TOKEN_INSTRUCTION_DISCRIMINATOR: [u8; 8] = [84, 52, 204, 228, 24, 140, 234, 75];
pub const BOOP_DEPLOY_BONDING_CURVE_INSTRUCTION_DISCRIMINATOR: [u8; 8] = [180, 89, 199, 76, 168, 236, 217, 138];

pub const BOOP_PROGRAM_ID: Pubkey = pubkey!("boop8hVGQGqehUK2iVEMEnMrL5RbjywRzHKBmBE7ry4");

pub enum BoopInstructionType {
    CreateToken,
    DeployBondingCurve,
}

#[derive(Debug, Default, Clone)]
pub struct CreateTokenParams {
    pub salt: u64,
    pub name: String,
    pub symbol: String,
    pub uri: String,
}

#[derive(Debug, Default, Clone)]
pub struct DeployBondingCurveParams {
    pub creator: Pubkey,
    pub salt: u64,
}

pub struct BoopDecoder;

impl InstructionDecoder for BoopDecoder {
    type InstructionType = BoopInstructionType;

    fn program_id() -> &'static Pubkey {
        &BOOP_PROGRAM_ID
    }

    fn protocol_name() -> &'static str {
        "Boop"
    }

    fn identify_instruction(data: &[u8]) -> Option<Self::InstructionType> {
         if data.len() < 8 {
            return None;
        }

        let discriminator = &data[0..8];

        match discriminator {
            d if d == BOOP_CREATE_TOKEN_INSTRUCTION_DISCRIMINATOR => Some(BoopInstructionType::CreateToken),
            d if d == BOOP_DEPLOY_BONDING_CURVE_INSTRUCTION_DISCRIMINATOR => Some(BoopInstructionType::DeployBondingCurve),
            _ => None,
        }
    }

    fn decode_instruction(
        instruction_type: Self::InstructionType,
        data: &[u8],
        accounts_indices: &[u8],
        account_keys: &[Pubkey],
        is_signer: &[bool],
        is_writable: &[bool],
    ) -> Result<JsonValue, String> {
        match instruction_type {
            BoopInstructionType::CreateToken => Self::decode_create_token(data, accounts_indices, account_keys, is_signer, is_writable),
            BoopInstructionType::DeployBondingCurve => Self::decode_deploy_bonding(data, accounts_indices, account_keys, is_signer, is_writable),
        }
    }
}

impl BoopDecoder {
    fn decode_create_token(
        data: &[u8],
        accounts_indices: &[u8],
        account_keys: &[Pubkey],
        is_signer: &[bool],
        is_writable: &[bool],
    ) -> Result<JsonValue, String> {
        let remaining_data = &data[8..];
        let mut offset = 0;

        let salt = parse_u64(remaining_data, &mut offset)?;
        let name = parse_string(remaining_data, &mut offset)?;
        let symbol = parse_string(remaining_data, &mut offset)?;
        let uri = parse_string(remaining_data, &mut offset)?;

        let parsed_data = object! {
            "salt" => salt.to_string(),
            "name" => name,
            "symbol" => symbol,
            "uri" => uri,
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
            parsed_data,
        ))
    }

    fn decode_deploy_bonding(
        data: &[u8],
        accounts_indices: &[u8],
        account_keys: &[Pubkey],
        is_signer: &[bool],
        is_writable: &[bool],
    ) -> Result<JsonValue, String> {
        let remaining_data = &data[8..];
        let mut offset = 0;

        let creator = parse_pubkey(remaining_data, &mut offset)?;
        let salt = parse_u64(remaining_data, &mut offset)?;

        let parsed_data = object! {
            "creator" => creator.to_string(),
            "salt" => salt.to_string(),
        };

        Ok(create_standardized_instruction(
            &BOOP_PROGRAM_ID,
            "DeployBondingCurve",
            "PumpAMM",
            data,
            accounts_indices,
            account_keys,
            is_signer,
            is_writable,
            parsed_data,
        ))
    }
}