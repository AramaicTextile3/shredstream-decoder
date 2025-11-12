use json::{object, JsonValue};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::pubkey;
use tracing::warn;
use std::convert::TryInto;

use crate::decoder::decoder::InstructionDecoder;
use crate::utils::create_standardized_instruction;

pub const PUMPAMM_PROGRAM_ID: Pubkey = pubkey!("pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA");

pub const PUMPAMM_CREATE_POOL_DISCRIMINATOR: [u8; 8] = [233, 146, 209, 142, 207, 104, 64, 188];
pub const PUMPAMM_CREATE_POOL_V2_DISCRIMINATOR: [u8; 8] = [214, 144, 76, 236, 95, 139, 49, 180];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PumpAmmInstructionType {
    CreatePool,
    CreatePoolV2,
}

pub struct PumpAmmDecoder;

impl InstructionDecoder for PumpAmmDecoder {
    type InstructionType = PumpAmmInstructionType;

    fn program_id() -> &'static Pubkey {
        &PUMPAMM_PROGRAM_ID
    }

    fn protocol_name() -> &'static str {
        "PumpAMM"
    }

    fn identify_instruction(data: &[u8]) -> Option<Self::InstructionType> {
        if data.len() < 8 {
            return None;
        }

        let discriminator = &data[0..8];

        match discriminator {
            d if d == PUMPAMM_CREATE_POOL_DISCRIMINATOR => Some(PumpAmmInstructionType::CreatePool),
            _ => None,
        }
    }

    fn decode_instruction(
        _instruction_type: Self::InstructionType,
        data: &[u8],
        accounts_indices: &[u8],
        account_keys: &[Pubkey],
        is_signer: &[bool],
        is_writable: &[bool],
    ) -> Result<JsonValue, String> {
        Self::decode_create_pool(data, accounts_indices, account_keys, is_signer, is_writable)
    }

    fn validate_account_count(instruction_type: &Self::InstructionType, account_count: usize) -> bool {
        let required = match instruction_type {
            PumpAmmInstructionType::CreatePool => 18,
            PumpAmmInstructionType::CreatePoolV2 => 18,
        };

        if account_count < required {
            warn!(
                "PumpAMM {:?} instruction: insufficient accounts (got {}, need {})",
                instruction_type, account_count, required
            );
            false
        } else {
            true
        }
    }
}

impl PumpAmmDecoder {
    fn decode_create_pool(
        data: &[u8],
        accounts_indices: &[u8],
        account_keys: &[Pubkey],
        is_signer: &[bool],
        is_writable: &[bool],
    ) -> Result<JsonValue, String> {
        // Validate minimum data length: 8 (discriminator) + 2 (u16) + 8 (u64) + 8 (u64) + 32 (Pubkey) = 58 bytes
        if data.len() < 58 {
            return Err("Insufficient data for CreatePool instruction".to_string());
        }

        let index = u16::from_le_bytes(
            data[8..10].try_into()
                .map_err(|_| "Failed to parse index")?
        );

        let base_amount_in = u64::from_le_bytes(
            data[10..18].try_into()
                .map_err(|_| "Failed to parse base_amount_in")?
        );

        let quote_amount_in = u64::from_le_bytes(
            data[18..26].try_into()
                .map_err(|_| "Failed to parse quote_amount_in")?
        );

        let coin_creator = Pubkey::new_from_array(
            data[26..58].try_into()
                .map_err(|_| "Failed to parse coin_creator")?
        );

        let parsed_data = object! {
            "index" => index.to_string(),
            "baseAmountIn" => base_amount_in.to_string(),
            "quoteAmountIn" => quote_amount_in.to_string(),
            "coinCreator" => bs58::encode(coin_creator.to_bytes()).into_string()
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
            parsed_data,
        ))
    }
}