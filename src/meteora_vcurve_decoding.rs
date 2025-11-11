use borsh::{BorshDeserialize, BorshSerialize};
use json::{object, JsonValue};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::pubkey;
use crate::utils::create_standardized_instruction;

pub const INITIALIZE_VIRTUAL_POOL_WITH_SPL_TOKEN_DISCRIMINATOR: [u8; 8] = [140, 85, 215, 176, 102, 54, 104, 79];

pub const METEORA_VCURVE_PROGRAM_ID: Pubkey = pubkey!("dbcij3LWUppWqq96dh6gJWwBifmcGfLSB5D4DuSMaqN");
pub const SYSTEM_PROGRAM_ID: Pubkey = pubkey!("11111111111111111111111111111111");
pub const TOKEN_PROGRAM_ID: Pubkey = pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
pub const METADATA_PROGRAM_ID: Pubkey = pubkey!("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s");
pub const POOL_AUTHORITY: Pubkey = pubkey!("FhVo3mqL8PW5pH5U2CN4XE33DokiyZnUwuGpH2hmHLuM");
pub const EVENT_AUTHORITY: Pubkey = pubkey!("8Ks12pbrD6PXxfty1hVQiE9sc289zgU1zHkvXhrSdriF");

pub enum MeteoraVCurveInstructionType {
    InitializeVirtualPoolWithSplToken,
}

#[derive(BorshDeserialize, BorshSerialize, Debug, Default, Clone)]
pub struct InitializePoolParameters {
    pub name: String,
    pub symbol: String,
    pub uri: String,
}

pub fn get_meteora_vcurve_instruction_type(data: &[u8]) -> Option<MeteoraVCurveInstructionType> {
    if data.len() < 8 {
        return None;
    }
    
    let discriminator = &data[0..8];
    
    match discriminator {
        d if d == INITIALIZE_VIRTUAL_POOL_WITH_SPL_TOKEN_DISCRIMINATOR => Some(MeteoraVCurveInstructionType::InitializeVirtualPoolWithSplToken),
        _ => None,
    }
}

pub fn deserialize_meteora_vcurve_initialize_virtual_pool_instruction(
    data: &[u8],
    accounts_indices: &[u8],
    account_keys: &[Pubkey],
    is_signer: &[bool],
    is_writable: &[bool],
) -> Result<JsonValue, String> {
    if data.len() < INITIALIZE_VIRTUAL_POOL_WITH_SPL_TOKEN_DISCRIMINATOR.len() {
        return Err("Data insufficient for an 'initialize_virtual_pool_with_spl_token' instruction.".to_string());
    }

    let remaining_data = &data[8..];
    let mut remaining_data_ref = &remaining_data[..];

    let params = match InitializePoolParameters::deserialize(&mut remaining_data_ref) {
        Ok(parsed) => parsed,
        Err(e) => {
            return Err(format!("Failed to deserialize InitializePoolParameters: {:?}", e));
        }
    };

    let parsed_data = object! {
        "name" => params.name,
        "symbol" => params.symbol,
        "uri" => params.uri,
    };
    
    Ok(create_standardized_instruction(
        &METEORA_VCURVE_PROGRAM_ID,
        "InitializeVirtualPoolWithSplToken",
        "MeteoraVCurve", 
        data,
        accounts_indices,
        account_keys,
        is_signer,
        is_writable,
        parsed_data
    ))
}
