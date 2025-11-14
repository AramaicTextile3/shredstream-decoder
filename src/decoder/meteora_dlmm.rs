use json::{object, JsonValue};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::{instruction, pubkey};
use tracing::warn;

use crate::decoder::InstructionDecoder;
use crate::decoder::byte_parser::{parse_bool, parse_i32, parse_option_u64, parse_u8, parse_u16, parse_u64, parse_vec};
use crate::utils::create_standardized_instruction;


// Initialize Pool IXs
const INIT_CUSTOMIZABLE_PERMISSIONLESS_LB_PAIR_DISCRIMINATOR: [u8; 8] = [46, 39, 41, 135, 111, 183, 200, 64];
const INIT_CUSTOMIZABLE_PERMISSIONLESS_LB_PAIR_2_DISCRIMINATOR: [u8; 8] = [243, 73, 129, 126, 51, 19, 241, 107];
const INIT_PERMISSION_LB_PAIR_DISCRIMINATOR: [u8; 8] = [108, 102, 213, 85, 251, 3, 53, 21];
const INIT_LB_PAIR_DISCRIMINATOR: [u8; 8] = [45, 154, 237, 210, 221, 15, 166, 92];
const INIT_LB_PAIR_2_DISCRIMINATOR: [u8; 8] = [73, 59, 36, 120, 237, 83, 108, 198];

// Add Liquidity IXs
const ADD_LIQUIDITY_DISCRIMINATOR: [u8; 8] = [181, 157, 89, 67, 143, 182, 52, 72];
const ADD_LIQUIDITY_2_DISCRIMINATOR: [u8; 8] = [228, 162, 78, 28, 70, 219, 116, 115];
const ADD_LIQUIDITY_BY_STRATEGY_DISCRIMINATOR: [u8; 8] = [7, 3, 150, 127, 148, 40, 61, 200];
const ADD_LIQUIDITY_BY_STRATEGY_2_DISCRIMINATOR: [u8; 8] = [3, 221, 149, 218, 111, 141, 118, 213];
const ADD_LIQUIDITY_BY_STRATEGY_ONE_SIDE_DISCRIMINATOR: [u8; 8] = [41, 5, 238, 175, 100, 225, 6, 205];
const ADD_LIQUIDITY_ONE_SIDE: [u8; 8] = [94, 155, 103, 151, 70, 95, 220, 165];
const ADD_LIQUIDITY_ONE_SIDE_PRECISE: [u8; 8] = [161, 194, 103, 84, 171, 71, 250, 154];
const ADD_LIQUIDITY_ONE_SIDE_PRECISE_2: [u8; 8] = [33, 51, 163, 201, 117, 98, 125, 231];
//const ADD_LIQUIDITY_BY_WEIGHT: [u8; 8] = [28, 140, 238, 99, 231, 162, 21, 149];

pub const METEORA_DLMM_PROGRAM: Pubkey = pubkey!("LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo");

#[derive(Debug)]
pub enum MeteoraDlmmInstructionType {
    InitCustomizablePermissionlessLbPair,
    InitCustomizablePermissionlessLbPair2,
    InitPermissionLbPair,
    InitLbPair,
    InitLbPair2,
    AddLiquidity,
    AddLiquidity2,
    AddLiquidityByStrategy,
    AddLiquidityByStrategy2,
    AddLiquidityByStrategyOneSide,
    AddLiquidityOneSide,
    AddLiquidityOneSidePrecise,
    AddLiquidityOneSidePrecise2,
}

pub struct MeteoraDlmmDecoder;

impl InstructionDecoder for MeteoraDlmmDecoder {
    type InstructionType = MeteoraDlmmInstructionType;

    fn program_id() -> &'static Pubkey {
        &METEORA_DLMM_PROGRAM
    }

    fn protocol_name() -> &'static str {
        "MeteoraDLMM"
    }

    fn identify_instruction(data: &[u8]) -> Option<Self::InstructionType> {
        if data.len() < 8 {
            return None;
        }

        let discriminator = &data[0..8];

        match discriminator {
            d if d == INIT_CUSTOMIZABLE_PERMISSIONLESS_LB_PAIR_DISCRIMINATOR => Some(MeteoraDlmmInstructionType::InitCustomizablePermissionlessLbPair),
            d if d == INIT_CUSTOMIZABLE_PERMISSIONLESS_LB_PAIR_2_DISCRIMINATOR => Some(MeteoraDlmmInstructionType::InitCustomizablePermissionlessLbPair2),
            d if d == INIT_PERMISSION_LB_PAIR_DISCRIMINATOR => Some(MeteoraDlmmInstructionType::InitPermissionLbPair),
            d if d == INIT_LB_PAIR_DISCRIMINATOR => Some(MeteoraDlmmInstructionType::InitLbPair),
            d if d == INIT_LB_PAIR_2_DISCRIMINATOR => Some(MeteoraDlmmInstructionType::InitLbPair2),
            d if d == ADD_LIQUIDITY_DISCRIMINATOR => Some(MeteoraDlmmInstructionType::AddLiquidity),
            d if d == ADD_LIQUIDITY_2_DISCRIMINATOR => Some(MeteoraDlmmInstructionType::AddLiquidity2),
            d if d == ADD_LIQUIDITY_BY_STRATEGY_DISCRIMINATOR => Some(MeteoraDlmmInstructionType::AddLiquidityByStrategy),
            d if d == ADD_LIQUIDITY_BY_STRATEGY_2_DISCRIMINATOR => Some(MeteoraDlmmInstructionType::AddLiquidityByStrategy2),
            d if d == ADD_LIQUIDITY_BY_STRATEGY_ONE_SIDE_DISCRIMINATOR => Some(MeteoraDlmmInstructionType::AddLiquidityByStrategyOneSide),
            d if d == ADD_LIQUIDITY_ONE_SIDE => Some(MeteoraDlmmInstructionType::AddLiquidityOneSide),
            d if d == ADD_LIQUIDITY_ONE_SIDE_PRECISE => Some(MeteoraDlmmInstructionType::AddLiquidityOneSidePrecise),
            d if d == ADD_LIQUIDITY_ONE_SIDE_PRECISE_2 => Some(MeteoraDlmmInstructionType::AddLiquidityOneSidePrecise2),
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
            MeteoraDlmmInstructionType::InitCustomizablePermissionlessLbPair | MeteoraDlmmInstructionType::InitCustomizablePermissionlessLbPair2 => Self::decode_init_customizable_permissionless_lb_pair(data, accounts_indices, account_keys, is_signer, is_writable),
            MeteoraDlmmInstructionType::InitPermissionLbPair => Self::decode_init_permission_lb_pair(data, accounts_indices, account_keys, is_signer, is_writable),
            MeteoraDlmmInstructionType::InitLbPair => Self::decode_init_lb_pair(data, accounts_indices, account_keys, is_signer, is_writable),
            MeteoraDlmmInstructionType::InitLbPair2 => Self::decode_init_lb_pair_2(data, accounts_indices, account_keys, is_signer, is_writable),
            MeteoraDlmmInstructionType::AddLiquidity | MeteoraDlmmInstructionType::AddLiquidity2 => Self::decode_add_liquidity(data, accounts_indices, account_keys, is_signer, is_writable),
            MeteoraDlmmInstructionType::AddLiquidityByStrategy => todo!(),
            MeteoraDlmmInstructionType::AddLiquidityByStrategy2 => todo!(),
            MeteoraDlmmInstructionType::AddLiquidityByStrategyOneSide => todo!(),
            MeteoraDlmmInstructionType::AddLiquidityOneSide => todo!(),
            MeteoraDlmmInstructionType::AddLiquidityOneSidePrecise => todo!(),
            MeteoraDlmmInstructionType::AddLiquidityOneSidePrecise2 => todo!(),
        }
    }
}

impl MeteoraDlmmDecoder {
    fn parse_accounts_type(data: &[u8], offset: &mut usize) -> Result<JsonValue, String> {
        let discriminant = parse_u8(data, offset)?;

        match discriminant {
            0 => Ok(object! { "type" => "TransferHookX" }),
            1 => Ok(object! { "type" => "TransferHookY" }),
            2 => Ok(object! { "type" => "TransferHookReward" }),
            3 => {
                let reward_index = parse_u8(data, offset)?;
                Ok(object! {
                    "type" => "TransferHookMultiReward",
                    "reward_index" => reward_index
                })
            }
            _ => Err(format!("Invalid AccountsType discriminant: {}", discriminant))
        }
    }

    fn parse_remaining_accounts_slice(data: &[u8], offset: &mut usize) -> Result<JsonValue, String> {
        let accounts_type = Self::parse_accounts_type(data, offset)?;
        let length = parse_u8(data, offset)?;

        Ok(object! {
            "accounts_type" => accounts_type,
            "length" => length,
        })
    }

    fn parse_remaining_accounts_info(data: &[u8], offset: &mut usize) -> Result<JsonValue, String> {
        let slices = parse_vec(
            data,
            offset,
            Self::parse_remaining_accounts_slice
        )?;

        Ok(object! {
            "slices" => slices,
        })
    }
    fn parse_bin_liquidity_distribution(data: &[u8], offset: &mut usize) -> Result<JsonValue, String> {
        let bin_id = parse_i32(data, offset)?;
        let distribution_x = parse_u16(data, offset)?;
        let distribution_y = parse_u16(data, offset)?;

        Ok(object! {
            "bin_id" => bin_id,
            "distribution_x" => distribution_x,
            "distribution_y" => distribution_y,
        })
    }

    fn decode_init_customizable_permissionless_lb_pair(
        data: &[u8],
        accounts_indices: &[u8],
        account_keys: &[Pubkey],
        is_signer: &[bool],
        is_writable: &[bool],
    ) -> Result<JsonValue, String> {
        let remaining_data = &data[8..];
        let mut offset = 0;

        let active_id = parse_i32(remaining_data, &mut offset)?;
        let bin_step = parse_u16(remaining_data, &mut offset)?;
        let base_factor = parse_u16(remaining_data, &mut offset)?;
        let activation_type = parse_u8(remaining_data, &mut offset)?;
        let has_alpha_vault = parse_bool(remaining_data, &mut offset)?;
        let activation_point = parse_option_u64(remaining_data, &mut offset)?;
        let creator_pool_on_off_control = parse_bool(remaining_data, &mut offset)?;
        let base_fee_power_factor = parse_u8(remaining_data, &mut offset)?;

        // padding, skip byte should work
        // if remaining_data.len() < offset + 62 {
        //     return Err("Insufficient data for padding".to_string());
        // }
        // offset += 62;

        let parsed_data = object! {
            "active_id" => active_id,
            "bin_step" => bin_step,
            "base_factor" => base_factor,
            "activation_type" => activation_type,
            "has_alpha_vault" => has_alpha_vault,
            "activation_point" => activation_point.map(|v| v.to_string()).unwrap_or("null".to_string()),
            "creator_pool_on_off_control" => creator_pool_on_off_control,
            "base_fee_power_factor" => base_fee_power_factor,
        };

        Ok(create_standardized_instruction(
            &METEORA_DLMM_PROGRAM,
            "initialize_customizable_permissionless_lb_pair",
            "MeteoraDLMM",
            data,
            accounts_indices,
            account_keys,
            is_signer,
            is_writable,
            parsed_data,
        ))
    }

    fn decode_init_permission_lb_pair(
        data: &[u8],
        accounts_indices: &[u8],
        account_keys: &[Pubkey],
        is_signer: &[bool],
        is_writable: &[bool],
    ) -> Result<JsonValue, String> {
        let remaining_data = &data[8..];
        let mut offset = 0;

        let active_id = parse_i32(remaining_data, &mut offset)?;
        let bin_step = parse_u16(remaining_data, &mut offset)?;
        let base_factor = parse_u16(remaining_data, &mut offset)?;
        let base_fee_power_factor = parse_u8(remaining_data, &mut offset)?;
        let activation_type = parse_u8(remaining_data, &mut offset)?;
        let protocol_share = parse_u16(remaining_data, &mut offset)?;


        let parsed_data = object! {
            "active_id" => active_id,
            "bin_step" => bin_step,
            "base_factor" => base_factor,
            "base_fee_power_factor" => base_fee_power_factor,
            "activation_type" => activation_type,
            "base_fee_power_factor" => base_fee_power_factor,
        };

        Ok(create_standardized_instruction(
            &METEORA_DLMM_PROGRAM,
            "initialize_permission_lb_pair",
            "MeteoraDLMM",
            data,
            accounts_indices,
            account_keys,
            is_signer,
            is_writable,
            parsed_data,
        ))
    }

    fn decode_init_lb_pair(
        data: &[u8],
        accounts_indices: &[u8],
        account_keys: &[Pubkey],
        is_signer: &[bool],
        is_writable: &[bool],
    ) -> Result<JsonValue, String> {
        let remaining_data = &data[8..];
        let mut offset = 0;

        let active_id = parse_i32(remaining_data, &mut offset)?;
        let bin_step = parse_u16(remaining_data, &mut offset)?;


        let parsed_data = object! {
            "active_id" => active_id,
            "bin_step" => bin_step,
        };

        Ok(create_standardized_instruction(
            &METEORA_DLMM_PROGRAM,
            "initialize_lb_pair",
            "MeteoraDLMM",
            data,
            accounts_indices,
            account_keys,
            is_signer,
            is_writable,
            parsed_data,
        ))
    }

    fn decode_init_lb_pair_2(
        data: &[u8],
        accounts_indices: &[u8],
        account_keys: &[Pubkey],
        is_signer: &[bool],
        is_writable: &[bool],
    ) -> Result<JsonValue, String> {
        let remaining_data = &data[8..];
        let mut offset = 0;

        let active_id = parse_i32(remaining_data, &mut offset)?;


        let parsed_data = object! {
            "active_id" => active_id,
        };

        // padding, skip byte should work
        // if remaining_data.len() < offset + 96 {
        //     return Err("Insufficient data for padding".to_string());
        // }
        // offset += 96;

        Ok(create_standardized_instruction(
            &METEORA_DLMM_PROGRAM,
            "initialize_permission_lb_pair_2",
            "MeteoraDLMM",
            data,
            accounts_indices,
            account_keys,
            is_signer,
            is_writable,
            parsed_data,
        ))
    }

    fn decode_add_liquidity(
        data: &[u8],
        accounts_indices: &[u8],
        account_keys: &[Pubkey],
        is_signer: &[bool],
        is_writable: &[bool],
    ) -> Result<JsonValue, String> {
        let discriminator = &data[0..8];
        let mut instruction_name = String::from("add_liquidity");
        let version = if discriminator == ADD_LIQUIDITY_2_DISCRIMINATOR {
            instruction_name = instruction_name + "2";
            2
        } else {
            1
        };

        let remaining_data = &data[8..];
        let mut offset = 0;

        let amount_x = parse_u64(remaining_data, &mut offset)?;
        let amount_y = parse_u64(remaining_data, &mut offset)?;

        let bin_liquidity_dist = parse_vec(
            remaining_data,
            &mut offset,
            Self::parse_bin_liquidity_distribution
        )?;

        let remaining_accounts_info = if version == 2 {
            Some(Self::parse_remaining_accounts_info(remaining_data, &mut offset)?)
        } else {
            None
        };

        let mut parsed_data = object! {
            "amount_x" => amount_x.to_string(),
            "amount_y" => amount_y.to_string(),
            "bin_liquidity_dist" => bin_liquidity_dist,
        };

        if let Some(info) = remaining_accounts_info {
            parsed_data["remaining_accounts_info"] = info;
        }

        Ok(create_standardized_instruction(
            &METEORA_DLMM_PROGRAM,
            &instruction_name,
            "MeteoraDLMM",
            data,
            accounts_indices,
            account_keys,
            is_signer,
            is_writable,
            parsed_data,
        ))
    }
}