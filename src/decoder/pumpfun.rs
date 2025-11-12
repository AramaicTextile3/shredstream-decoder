use borsh::{BorshDeserialize, BorshSerialize};
use json::{object, JsonValue};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::pubkey;
use tracing::{error, warn};

use crate::decoder::byte_parser::{parse_pubkey, parse_string, parse_bool};
use crate::decoder::decoder::InstructionDecoder;
use crate::utils::create_standardized_instruction;

pub struct PumpfunDecoder;

pub const PUMP_CREATE_INSTRUCTION_DISCRIMINATOR: [u8; 8] = [24, 30, 200, 40, 5, 28, 7, 119];
pub const PUMP_CREATE_V2_INSTRUCTION_DISCRIMINATOR: [u8; 8] = [214, 144, 76, 236, 95, 139, 49, 180];

pub const PUMP_MIGRATION_PROGRAM: Pubkey = pubkey!("39azUYFWPz3VHgKCf3VChUwbpURdCHRxjWVowf5jUJjg");
pub const PUMPFUN_PROGRAM_ID: Pubkey = pubkey!("6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P");
pub const PUMPFUN_MINT_AUTHORITY: Pubkey = pubkey!("TSLvdd1pWpHVjahSpsvCXUbgwsL3JAcvokwaKt1eokM");
pub const PUMPFUN_GLOBAL: Pubkey = pubkey!("4wTV1YmiEkRvAtNtsSGPtUrqRYQMe5SKy2uB4Jjaxnjf");
pub const TOKEN_PROGRAM_ID: Pubkey = pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");

#[derive(Debug)]
pub enum PumpfunInstructionType {
    Create,
    CreateV2,
}

impl InstructionDecoder for PumpfunDecoder {
    type InstructionType = PumpfunInstructionType;

    fn program_id() -> &'static Pubkey {
        &PUMPFUN_PROGRAM_ID
    }

    fn protocol_name() -> &'static str {
        "Pumpfun"
    }

    fn identify_instruction(data: &[u8]) -> Option<Self::InstructionType> {
        if data.len() < 8 {
            return None;
        }

        let discriminator = &data[0..8];

        match discriminator {
            d if d == PUMP_CREATE_INSTRUCTION_DISCRIMINATOR => Some(PumpfunInstructionType::Create),
            d if d == PUMP_CREATE_V2_INSTRUCTION_DISCRIMINATOR => Some(PumpfunInstructionType::CreateV2),
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
            PumpfunInstructionType::Create => Self::decode_create(data, accounts_indices, account_keys, is_signer, is_writable),
            PumpfunInstructionType::CreateV2 => Self::decode_create_v2(data, accounts_indices, account_keys, is_signer, is_writable),
        }
    }

    fn validate_account_count(instruction_type: &Self::InstructionType, account_count: usize) -> bool {
        let required = match instruction_type {
            PumpfunInstructionType::Create => 18,
            PumpfunInstructionType::CreateV2 => 18,
        };

        if account_count < required {
            warn!(
                "PumpFun {:?} instruction: insufficient accounts (got {}, need {})",
                instruction_type, account_count, required
            );
            false
        } else {
            true
        }
    }
}

impl PumpfunDecoder {
    fn decode_create(
        data: &[u8],
        accounts_indices: &[u8],
        account_keys: &[Pubkey],
        is_signer: &[bool],
        is_writable: &[bool],
    ) -> Result<JsonValue, String> {
        let remaining_data = &data[8..];
        let mut offset = 0;
        
        let name = parse_string(remaining_data, &mut offset)?;
        let symbol = parse_string(remaining_data, &mut offset)?;
        let uri = parse_string(remaining_data, &mut offset)?;
        let creator = parse_pubkey(remaining_data, &mut offset)?;


        let parsed_data = object! {
            "name" => name,
            "symbol" => symbol,
            "uri" => uri,
            "creator" => creator.to_string()
        };

        Ok(create_standardized_instruction(
            &PUMPFUN_PROGRAM_ID,
            "create",
            "Pumpfun",
            data,
            accounts_indices,
            account_keys,
            is_signer,
            is_writable,
            parsed_data,
        ))
    }

    fn decode_create_v2(
        data: &[u8],
        accounts_indices: &[u8],
        account_keys: &[Pubkey],
        is_signer: &[bool],
        is_writable: &[bool],
    ) -> Result<JsonValue, String> {
        let remaining_data = &data[8..];
        let mut offset = 0;
        
        let name = parse_string(remaining_data, &mut offset)?;
        let symbol = parse_string(remaining_data, &mut offset)?;
        let uri = parse_string(remaining_data, &mut offset)?;
        let creator = parse_pubkey(remaining_data, &mut offset)?;
        let is_mayhem_mode = parse_bool(remaining_data, &mut offset)?;


        let parsed_data = object! {
            "name" => name,
            "symbol" => symbol,
            "uri" => uri,
            "creator" => creator.to_string(),
            "is_mayhem_mode" => is_mayhem_mode,
        };

        Ok(create_standardized_instruction(
            &PUMPFUN_PROGRAM_ID,
            "create_v2",
            "Pumpfun",
            data,
            accounts_indices,
            account_keys,
            is_signer,
            is_writable,
            parsed_data,
        ))
    }
}