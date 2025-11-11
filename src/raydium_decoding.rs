use borsh::{BorshDeserialize, BorshSerialize};
use json::{object, JsonValue};
use solana_sdk::pubkey;
use solana_sdk::pubkey::Pubkey;
use crate::utils::create_standardized_instruction;

pub const RAYDIUM_LP_PROGRAM: Pubkey = pubkey!("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8");
pub const SYSTEM_PROGRAM_ID: Pubkey = pubkey!("11111111111111111111111111111111");
pub const TOKEN_PROGRAM_ID: Pubkey = pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
pub const ASSOCIATED_TOKEN_PROGRAM: Pubkey =
    pubkey!("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");
pub const RENT_PROGRAM: Pubkey = pubkey!("SysvarRent111111111111111111111111111111111");
pub const RAYDIUM_AUTHORITY: Pubkey = pubkey!("5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1");
pub const WSOL: Pubkey = pubkey!("So11111111111111111111111111111111111111112");
pub const SERUM_PROGRAM_OPENBOOK: Pubkey = pubkey!("srmqPvymJeFKQ4zGQed1GFppgkRHL9kaELCbyksJtPX");

#[derive(BorshDeserialize, BorshSerialize, Debug, Default, Clone)]
pub struct Initialize2Params {
    discriminator: u8,
    nonce: u8,
    open_time: u64,
    init_pc_amount: u64,
    init_coin_amount: u64,
}

pub enum RaydiumInstructionType {
    Initialize2,
}

pub fn get_raydium_instruction_type(data: &[u8]) -> Option<RaydiumInstructionType> {
    match data.get(0..1) {
        Some(d) if d[0] == 1 => Some(RaydiumInstructionType::Initialize2),
        _ => None,
    }
}

pub fn deserialize_raydium_initialize2_instruction(
    data: &[u8],
    accounts_indices: &[u8],
    account_keys: &[Pubkey],
    is_signer: &[bool],
    is_writable: &[bool],
) -> Result<JsonValue, String> {
    let mut data_ref = &data[0..];

    let args = match Initialize2Params::deserialize(&mut data_ref) {
        Ok(parsed) => parsed,
        Err(e) => {
            return Err(format!("Failed to deserialize Initialize2Params: {:?}", e));
        }
    };
    
    let parsed_data = object! {
        "discriminator" => args.discriminator.to_string(),
        "nonce" => args.nonce.to_string(),
        "open_time" => args.open_time.to_string(),
        "init_pc_amount" => args.init_pc_amount.to_string(),
        "init_coin_amount" => args.init_coin_amount.to_string(),
    };
    
    Ok(create_standardized_instruction(
        &RAYDIUM_LP_PROGRAM,
        "Initialize2",
        "Raydium", 
        data,
        accounts_indices,
        account_keys,
        is_signer,
        is_writable,
        parsed_data
    ))
}