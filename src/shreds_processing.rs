use crate::moonit_decoding::*;
use crate::pumpfun_decoding::*;
use crate::pumpamm_decoding::*;
use crate::raydium_decoding::*;
use crate::raydium_launchlab_decoding::*;
use crate::raydium_cpmm_decoding::*;
use crate::meteora_vcurve_decoding::*;
use crate::boop_decoding::*;
use crate::meteoradyn_decoding::*;
use crate::meteora_amm_v2_decoding::*;
use crate::orca_decoding::*;
use crate::utils::*;
use crate::address_lookup_table_cache::*;

use dashmap::{DashMap, DashSet};
use json::{object, JsonValue};
use rayon::prelude::*;
use rustc_hash::FxHashMap as HashMap;
use solana_entry::entry::Entry;
use solana_ledger::shred::{ReedSolomonCache, Shred, ShredType, Shredder};
use solana_sdk::message::VersionedMessage;
use solana_sdk::signature::SIGNATURE_BYTES;
use solana_sdk::transaction::VersionedTransaction;
use std::convert::TryInto;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use thiserror::Error;
// For logging
use tracing::{debug, error, info, warn};

const SIZE_OF_COMMON_SHRED_HEADER: usize = 83;
const SIZE_OF_SIGNATURE: usize = SIGNATURE_BYTES;
const SIZE_OF_SHRED_VARIANT: usize = 1;
const SIZE_OF_SHRED_SLOT: usize = 8;
const OFFSET_OF_SHRED_SLOT: usize = SIZE_OF_SIGNATURE + SIZE_OF_SHRED_VARIANT;

#[derive(Error, Debug)]
pub enum FecBlockError {
    #[error("FullPayloadReconstructionError: {0}")]
    FullPayloadReconstructionError(String),

    #[error("Slot mismatch: expected slot={expected}, but found slot={found}")]
    SlotMismatch { expected: u64, found: u64 },
}

#[derive(Debug, thiserror::Error)]
pub enum CollectShredsError {
    #[error("FecBlockError: {0}")]
    FecBlockError(#[from] FecBlockError),

    #[error("{0}")]
    GeneralError(String),
}

#[allow(dead_code)]
pub struct CodingShredHeader {
    pub num_data_shreds: u16,
    pub num_coding_shreds: u16,
    pub position: u16,
}

#[derive(Debug, Clone)]
pub struct FecBlock {
    pub num_data_shreds: Option<u16>,
    pub num_coding_shreds: Option<u16>,
    pub data_shreds_collected: usize,
    pub coding_shreds_collected: usize,
    pub data_shreds: HashMap<u32, Shred>,
    pub coding_shreds: HashMap<u32, Shred>,
    pub fec_set_index: u32,
    pub slot: u64,
    pub collection_start: Option<Instant>,
    pub last_shred_in_slot: bool,
}

impl FecBlock {
    // Creating a new FecBlock structure
    pub fn new(slot: u64, fec_set_index: u32) -> Self {
        FecBlock {
            num_data_shreds: None,
            num_coding_shreds: None,
            data_shreds_collected: 0,
            coding_shreds_collected: 0,
            data_shreds: HashMap::default(),
            coding_shreds: HashMap::default(),
            fec_set_index,
            slot,
            collection_start: None,
            last_shred_in_slot: false,
        }
    }

    // Function that checks if FecBlock is complete
    pub fn is_complete(&self, processed_blocks: &Arc<DashSet<(u64, u32)>>) -> bool {
        if let (Some(expected_data), Some(expected_coding)) =
            (self.num_data_shreds, self.num_coding_shreds)
        {
            let data_count = self.data_shreds.len();
            let coding_count = self.coding_shreds.len();
            let total_shreds = data_count + coding_count;

            let total_expected = expected_data as usize + expected_coding as usize;
            let complete =
                (data_count == expected_data as usize) || (total_shreds >= total_expected);

            if complete {
                processed_blocks.insert((self.slot, self.fec_set_index));
                debug!(
                    "FecBlock {} for slot {} has been processed and added to DashSet.",
                    self.fec_set_index, self.slot
                );
            }
            complete
        } else {
            false
        }
    }
}

// Collects data and coding shreds for the FEC block
pub async fn collect_shred(
    shred_data: &[u8], 
    fec_blocks: &Arc<DashMap<(u64, u32), FecBlock>>,
    processed_blocks: &Arc<DashSet<(u64, u32)>>, 
    broadcast_tx: tokio::sync::broadcast::Sender<(String, u64)>,
) -> Result<(), CollectShredsError> {
    debug!("collect_shred: Top of function");
    if shred_data.len() < SIZE_OF_COMMON_SHRED_HEADER {
        return Ok(());
    }
    let shred_collect_start = Instant::now();

    // let fec_set_index = match get_fec_set_index_from_data(shred_data) {
    //     Ok(value) => value,
    //     Err(err) => {
    //         return Err(CollectShredsError::GeneralError(format!("Failed to extract fec_set_index from received shred_data: {}", err))); 
    //     }
    // };
        
    // let shred_slot = match get_slot_from_shred_data(shred_data) {
    //     Ok(value) => value,
    //     Err(err) => {
    //         return Err(CollectShredsError::GeneralError(format!("Failed to extract slot from received shred_data: {}", err)));
    //     }
    // };
        
        
    let shred = Shred::new_from_serialized_shred(Vec::from(shred_data))
        .map_err(|e| CollectShredsError::GeneralError(format!("Error creating Shred object: {:?}", e)))?;

    let fec_set_index = shred.fec_set_index();
    let shred_slot = shred.slot();

    // We don't create the Shred object if we don't pass this check
    if processed_blocks.contains(&(shred_slot, fec_set_index)) {
        debug!("Skipping FecBlock {} in slot {} as it is already processed.", fec_set_index, shred_slot);
        return Ok(());
    }

    let shred_type = shred.shred_type();
    let shred_index = shred.index();

    // Checking if shred index is valid
    if shred_index < fec_set_index {
        return Err(CollectShredsError::GeneralError(format!("Shred index {} < fec_set_index {}", shred_index, fec_set_index)));
    }
    debug!("\n═════════════════════════════════════════════════════════════════════════════════════════");
    // Add the shred to the FecBlock
    add_shred(
        shred, 
        fec_blocks,
        processed_blocks, 
        shred_type, 
        shred_index, 
        fec_set_index, 
        shred_slot, 
        broadcast_tx,
    )?;

    debug!("Info from Shred and creation of the object took: {:?}", shred_collect_start.elapsed());
    Ok(())
}

// Function that adds a shred to the FEC block
pub fn add_shred( 
    shred: Shred, 
    fec_blocks: &Arc<DashMap<(u64, u32), FecBlock>>,
    processed_blocks: &Arc<DashSet<(u64, u32)>>,
    shred_type: ShredType, 
    shred_index: u32, 
    fec_set_index: u32,  
    shred_slot: u64, 
    broadcast_tx: tokio::sync::broadcast::Sender<(String, u64)>,
) -> Result<(), FecBlockError> {
    let start_total = Instant::now(); // For debugging, to be removed in production

    let mut should_decode = false;
    let key = (shred_slot, fec_set_index);
    // Get or create FecBlock using slot and fec_set_index   
    
    // if processed_blocks.contains(&key) {
    //     debug!("FecBlock {} from slot {} as was already processed.", fec_set_index, shred_slot);
    //     return Ok(());
    // }
   
    let mut fec_block = fec_blocks.entry((shred_slot, fec_set_index))
        .or_insert_with(|| FecBlock::new(shred_slot, fec_set_index));

    if shred_slot == fec_block.slot {
        match shred_type {
            ShredType::Code => {
                if fec_block.coding_shreds.contains_key(&shred_index) {
                    debug!("Code Shred shred_index={} fec_set_index={} from slot={} is already colected", shred_index, fec_block.slot, fec_block.fec_set_index);
                    return Ok(());
                }
                let shred_payload = shred.payload();
                if fec_block.num_data_shreds.is_none() || fec_block.num_coding_shreds.is_none()
                {
                    if let Ok(CodingShredHeader {
                        num_data_shreds,
                        num_coding_shreds,
                        position: _,
                    }) = get_coding_shred_header(shred_payload)
                    {
                        fec_block.num_data_shreds = Some(num_data_shreds);
                        fec_block.num_coding_shreds = Some(num_coding_shreds);
                    } 
                }
                fec_block.coding_shreds.insert(shred_index, shred); // Adding the Coding shred
                fec_block.coding_shreds_collected += 1;
            }
            ShredType::Data => {
                if fec_block.data_shreds.contains_key(&shred_index) {
                    debug!("Data Shred shred_index={} fec_set_index={} from slot={} is already colected", shred_index, fec_block.slot, fec_block.fec_set_index);
                    return Ok(());
                }
                if shred.last_in_slot() && shred.data_complete() {
                    fec_block.last_shred_in_slot = true;
                    debug!(
                        "Last FecBlock {} detected for slot {} (shred_index: {}).",
                        fec_set_index, shred_slot, shred_index
                    );
                }
                fec_block.data_shreds.insert(shred_index, shred); // Adding the Data shred
                fec_block.data_shreds_collected += 1;
            }
        };
    } else {
        return Err(FecBlockError::SlotMismatch {expected: fec_block.slot, found: shred_slot,});
    }

    if fec_block.collection_start.is_none() {
        fec_block.collection_start = Some(Instant::now());
        debug!("Initializing collection_start for FEC Block {}: FEC Block Slot: {}.", fec_block.fec_set_index, fec_block.slot);
    }
    debug!("\n[{:?} - SHRED COLLECTED]: SLOT: {:?} FEC_SET_INDEX: {}, SHRED_INDEX: {}
                                SHREDS COLLECTED: Data: {} / {} - Coding: {} / {}\n",
        shred_type, shred_slot, fec_set_index, shred_index, fec_block.data_shreds.len(), fec_block.num_data_shreds.unwrap_or(0), fec_block.coding_shreds.len(), fec_block.num_coding_shreds.unwrap_or(0)
    );

    // Garbage collector to remove the FecBlocks that are too old and not completed
    if fec_block.collection_start.map_or(false, |start| start.elapsed() > Duration::from_secs(30)) {
        debug!("FecBlock expired, removing: slot {} fec_set_index {}", shred_slot, fec_set_index);
        fec_blocks.remove(&(shred_slot, fec_set_index));
        increment_slot_counters(shred_slot, 1, 0, 0, 1);
        // save_slot_statistics_to_file(shred_slot);
    }        

    if fec_block.is_complete(processed_blocks) {
        should_decode = true;
    }
    drop(fec_block);

    if should_decode {
        let fec_blocks = Arc::clone(fec_blocks);
        let broadcast_tx = broadcast_tx.clone();

        // Use tokio::spawn_blocking for CPU-bound task to maintain async context
        tokio::task::spawn_blocking(move || {
            let fec_block_ref = match fec_blocks.get(&key) {
                Some(entry) => entry.clone(),
                None => {
                    debug!("FecBlock for key {:?} was not found in DashMap", key);
                    return;
                }
            };

            if let Some(start_time) = fec_block_ref.collection_start {
                let collection_duration = start_time.elapsed();      

                info!(
                    "FEC Block complete for slot {}, fec_set_index {} in {:?}",
                    fec_block_ref.slot, fec_block_ref.fec_set_index, collection_duration,
                );
                
                let start_processing = Instant::now();

                let decoded_fec_set_index = fec_block_ref.fec_set_index;
                let decoded_slot = fec_block_ref.slot;
                let _last_shred_in_slot = fec_block_ref.last_shred_in_slot;
                
                match decode_fec_block(&fec_block_ref) {
                    Ok((reconstructed_payload, slot)) => {
                        let (_, tx_count) = tokio::task::block_in_place(|| {
                            tokio::runtime::Handle::current().block_on(async {
                                extract_transactions_from_payload(
                                    slot, 
                                    reconstructed_payload,
                                    broadcast_tx,
                                ).await
                            })
                        });
                        let _start_slot_stats_updates = Instant::now();
                        increment_slot_counters(slot, 1, tx_count as u64, 1, 0);
                        // if last_shred_in_slot {
                        //     save_slot_statistics_to_file(slot);
                        //     info!("Statistics for slot {} has been saved into json file in {:?}", decoded_slot, start_slot_stats_updates.elapsed());
                        // }
                        debug!("Finalized decode for FecBlock {} slot = {} in {:?}", 
                            decoded_fec_set_index, decoded_slot, start_processing.elapsed()
                        );
                    }
                    Err(e) => {
                        error!("DecodeFecBlockError: slot={}, fec_set_index={}: {:?}", 
                            decoded_slot, 
                            decoded_fec_set_index,
                            e
                        );
                    }
                }
                fec_blocks.remove(&key);
                debug!("FecBlock {} for slot {} has been processed and removed from DashMap.", decoded_fec_set_index, decoded_slot);
            }
        });
    }
    debug!("Total add_shred function duration: {:?}", start_total.elapsed());

    Ok(())
}

// Function that decodes the FecBlock without modifying the original FecBlock
pub fn decode_fec_block(
    fec_block: &FecBlock,
) -> Result<(Vec<u8>, u64), FecBlockError> {

    let mut local_data_shreds: Vec<Shred> = fec_block.data_shreds.values().cloned().collect();
    let expected_data_shreds = fec_block.num_data_shreds.unwrap_or(1) as usize;

    if local_data_shreds.len() < expected_data_shreds {
        info!(
            "Attempting to recover missing data shreds from slot {}, fec_set {}",
            fec_block.slot, fec_block.fec_set_index
        );
        let all_shreds_for_recovery: Vec<Shred> = fec_block.data_shreds
            .values()
            .cloned()
            .chain(fec_block.coding_shreds.values().cloned())
            .collect();

        match Shredder::try_recovery(all_shreds_for_recovery, &ReedSolomonCache::default()) {
            Ok(recovered_shreds) => {
                let mut count_recovered = 0;
                for recovered_shred in recovered_shreds.into_iter().filter(|s| s.is_data()) {
                    local_data_shreds.push(recovered_shred);
                    count_recovered += 1;
                }
                info!(
                    "Recovered {} data shreds for slot {} FEC set {}",
                    count_recovered, fec_block.slot, fec_block.fec_set_index
                );
            }
            Err(e) => {
                warn!("Failed to recover data shreds: {:?}", e);
            }
        }
    }

    if local_data_shreds.is_empty() {
        error!("No data shreds after attempt to recover");
    }

    local_data_shreds.sort_by_key(|shred| shred.index());

    let payload = reconstruct_full_payload(&local_data_shreds)
        .map_err(|e| FecBlockError::FullPayloadReconstructionError(e))?;

    Ok((payload, fec_block.slot))
}

fn reconstruct_full_payload(shreds: &[Shred]) -> Result<Vec<u8>, String> {
    let shred_payloads: Vec<Vec<u8>> = shreds.iter()
        .map(|shred| shred.payload().to_vec())
        .collect();
    
    Shredder::deshred(shred_payloads).map_err(|e| {
        format!("Failed to deshred payload: {:?}", e)
    })
}

// Function to extract fec_set_index from the shred data
fn get_fec_set_index_from_data(shred_data: &[u8]) -> Result<u32, &'static str> {
    if shred_data.len() < SIZE_OF_COMMON_SHRED_HEADER { 
        return Err("shred is too short.");
    }
    let fec_set_index_bytes = &shred_data[79..79 + 4]; // Offset 79 for fec_set_index
    match fec_set_index_bytes.try_into() {
        Ok(bytes) => Ok(u32::from_le_bytes(bytes)),
        Err(_) => {
            Err("Error converting fec_set_index.")
        }
    }
}

fn get_coding_shred_header(shred_data: &[u8]) -> Result<CodingShredHeader, Box<dyn std::error::Error>> {
    Ok(CodingShredHeader {
        num_data_shreds: u16::from_le_bytes(
            shred_data[0x53..0x55].try_into()?,
        ),
        num_coding_shreds: u16::from_le_bytes(
            shred_data[0x55..0x57].try_into()?,
        ),
        position: u16::from_le_bytes(shred_data[0x57..0x59].try_into()?),
    })
}

fn get_slot_from_shred_data(shred_data: &[u8]) -> Result<u64, &'static str> {
    if shred_data.len() < OFFSET_OF_SHRED_SLOT + SIZE_OF_SHRED_SLOT {
        return Err("The shred is too short");
    }
    let slot_bytes = &shred_data[OFFSET_OF_SHRED_SLOT..OFFSET_OF_SHRED_SLOT + SIZE_OF_SHRED_SLOT];
    let slot = u64::from_le_bytes(slot_bytes.try_into().unwrap());
    Ok(slot)
}

pub async fn extract_transactions_from_payload(
    slot: u64,
    payload: Vec<u8>,
    broadcast_tx: tokio::sync::broadcast::Sender<(String, u64)>,
) -> (u64, usize) {
    match bincode::deserialize::<Vec<Entry>>(&payload) {
        Ok(entries) => {
            // Pre-resolve all address lookup tables before parallel processing
            let mut transactions_with_resolved_keys = Vec::new();
            
            for entry in &entries {
                for transaction in &entry.transactions {
                    // Process transaction for lookup table extensions first
                    crate::address_lookup_table_cache::process_transaction_for_lookup_table_extensions(transaction);
                    
                    let (base_account_keys, address_table_lookups) = match &transaction.message {
                        VersionedMessage::Legacy(legacy_msg) => (
                            &legacy_msg.account_keys,
                            None,
                        ),
                        VersionedMessage::V0(v0_msg) => (
                            &v0_msg.account_keys,
                            Some(v0_msg.address_table_lookups.as_slice()),
                        ),
                    };
                    
                    // Resolve account keys asynchronously while we have Tokio context
                    let resolved_account_keys = resolve_transaction_account_keys(
                        base_account_keys, 
                        address_table_lookups
                    ).await;
                    
                    transactions_with_resolved_keys.push((transaction, resolved_account_keys));
                }
            }
            
            // Now do parallel processing with pre-resolved account keys
            let total_txs = transactions_with_resolved_keys
                .par_iter()
                .map(|(transaction, resolved_account_keys)| {
                    let start_time = Instant::now();

                    if let Some(json_transaction) = deserialize_versioned_transaction_with_resolved_keys(transaction, slot, resolved_account_keys) {
                        let timestamp = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .expect("Time went backwards")
                            .as_micros();

                        if let Err(e) = broadcast_tx.send((json_transaction.pretty(2).to_string(), timestamp as u64)) {
                            error!("Failed to send transaction via grpc broadcast channel: {}", e);
                        }
                        debug!("Transaction deserialized & sent to channel in {:?}", start_time.elapsed());
                    }
                    
                    1  // Count each transaction
                })
                .sum();

            (slot, total_txs)
        }
        Err(e) => {
            error!("Error deserializing Entries from payload: {:?}", e);
            (slot, 0)
        }
    }
}

// Note: resolve_account_keys_sync function removed - address resolution now happens
// before parallel processing in extract_transactions_from_payload

fn deserialize_versioned_transaction_with_resolved_keys(transaction: &VersionedTransaction, slot: u64, resolved_account_keys: &[solana_sdk::pubkey::Pubkey]) -> Option<JsonValue> {
    let (instructions, _base_account_keys, header, recent_blockhash, _address_table_lookups) =
        match &transaction.message {
        VersionedMessage::Legacy(legacy_msg) => (
            &legacy_msg.instructions,
            &legacy_msg.account_keys,
            &legacy_msg.header,
            &legacy_msg.recent_blockhash,
            None,
        ),
        VersionedMessage::V0(v0_msg) => (
            &v0_msg.instructions,
            &v0_msg.account_keys,
            &v0_msg.header,
            &v0_msg.recent_blockhash,
            Some(&v0_msg.address_table_lookups),
        ),
    };

    // Use the pre-resolved account keys passed as parameter
    let account_keys = resolved_account_keys;
    
    let num_accounts = account_keys.len();
    let mut is_signer = vec![false; num_accounts];
    let mut is_writable = vec![false; num_accounts];
    
    for i in 0..header.num_required_signatures as usize {
        if i < num_accounts {
            is_signer[i] = true;
        }
    }
    
    for i in 0..(header.num_required_signatures - header.num_readonly_signed_accounts) as usize {
        if i < num_accounts {
            is_writable[i] = true;
        }
    }
    
    for i in (header.num_required_signatures + header.num_readonly_unsigned_accounts) as usize..num_accounts {
        is_writable[i] = true;
    }

    let mut serialized_instructions = Vec::with_capacity(instructions.len());
    let mut contains_relevant_instruction = false;

    for instr in instructions.iter() {
        let Some(program_key) = account_keys.get(instr.program_id_index as usize) else { continue };
        if instr.data.len() < 8 { continue };

        if program_key == &PUMPFUN_PROGRAM_ID {
            let Some(instr_type) = get_pumpfun_instruction_type(&instr.data) else { continue };
            contains_relevant_instruction = true;
                
            let decoded_result = match instr_type {
                PumpfunInstructionType::Create => {
                    if instr.accounts.len() < 14 {
                        warn!("Pumpfun Create: The instruction does not contain a minimum of 14 accounts needed.");
                        continue;
                    }
                    deserialize_pump_create_instruction(&instr.data, &instr.accounts, account_keys, &is_signer, &is_writable)
                },
                PumpfunInstructionType::Buy => {
                    continue;
                },
            };

            match decoded_result {
                Ok(decoded) => serialized_instructions.push(decoded),
                Err(err) => error!("Error decoding Pumpfun instruction: {}", err),
            }
        } else if program_key == &RAYDIUM_LP_PROGRAM {
            let Some(instr_type) = get_raydium_instruction_type(&instr.data) else { continue };
            contains_relevant_instruction = true;
                
            let decoded_result = match instr_type {
                RaydiumInstructionType::Initialize2 => {
                if instr.accounts.len() < 21 {
                    warn!("Raydium Initialize2: The instruction does not contain a minimum of 21 accounts needed.");
                    continue;
                }
                    deserialize_raydium_initialize2_instruction(&instr.data, &instr.accounts, account_keys, &is_signer, &is_writable)
                }
            };

            match decoded_result {
                Ok(decoded) => serialized_instructions.push(decoded),
                Err(err) => error!("Error decoding Raydium instruction: {}", err),
            }
        } else if program_key == &MOONIT_PROGRAM_ID {
            let Some(instr_type) = get_moonit_instruction_type(&instr.data) else { continue };
            contains_relevant_instruction = true;
                
            let decoded_result = match instr_type {
                MoonitInstructionType::TokenMint => {
                    if instr.accounts.len() < 11 {
                        warn!("Moonit TokenMint: The instruction does not contain a minimum of 11 accounts needed.");
                        continue;
                    }
                    deserialize_moonit_token_mint_instruction(&instr.data, &instr.accounts, account_keys, &is_signer, &is_writable)
                }
            };

            match decoded_result {
                Ok(decoded) => serialized_instructions.push(decoded),
                Err(err) => error!("Error decoding Moonit instruction: {}", err),
            }
        } else if program_key == &RAYDIUM_LAUNCHLAB_PROGRAM_ID {
            let Some(instr_type) = get_raydium_launchlab_instruction_type(&instr.data) else { continue };
            contains_relevant_instruction = true;
            
            let decoded_result = match instr_type {
                RaydiumLaunchlabInstructionType::Initialize => {
                    if instr.accounts.len() < 18 {
                        warn!("Raydium Launchlab Initialize: The instruction does not contain a minimum of 18 accounts needed.");
                        continue;
                    }
                    deserialize_raydium_launchlab_initialize_instruction(&instr.data, &instr.accounts, account_keys, &is_signer, &is_writable)
                }
            };
            match decoded_result {
                Ok(decoded) => serialized_instructions.push(decoded),
                Err(err) => {
                    error!("Error decoding Raydium Launchlab instruction: {}", err);
                    continue;
                }
            }
        } else if program_key == &BOOP_PROGRAM_ID {
            let Some(instr_type) = get_boop_instruction_type(&instr.data) else { continue };
            contains_relevant_instruction = true;
                
            let decoded_result = match instr_type {
                BoopInstructionType::CreateToken => {
                    if instr.accounts.len() < 8 {
                        warn!("Boop CreateToken: The instruction does not contain a minimum of 8 accounts needed.");
                        continue;
                    }
                    deserialize_boop_create_token_instruction(&instr.data, &instr.accounts, account_keys, &is_signer, &is_writable)
                },
                BoopInstructionType::DeployBondingCurve => {
                    if instr.accounts.len() < 10 {
                        warn!("Boop DeployBondingCurve: The instruction does not contain a minimum of 10 accounts needed.");
                        continue;
                    }
                    deserialize_boop_deploy_bonding_curve_instruction(&instr.data, &instr.accounts, account_keys, &is_signer, &is_writable)
                }
            };

            match decoded_result {
                Ok(decoded) => serialized_instructions.push(decoded),
                Err(err) => {
                    error!("Error decoding Boop instruction: {}", err);
                    continue;
                }
            }
        } else if program_key == &PUMPAMM_PROGRAM_ID {
            let Some(instr_type) = get_pumpamm_instruction_type(&instr.data) else { continue };
            contains_relevant_instruction = true;
                
            let decoded_result = match instr_type {
                PumpAmmInstructionType::Buy => {
                    if instr.accounts.len() < 19 {
                        warn!("PumpAMM Buy: The instruction does not contain a minimum of 16 accounts needed.");
                        continue;
                    }
                    deserialize_pumpamm_buy_instruction(&instr.data, &instr.accounts, account_keys, &is_signer, &is_writable)
                },
                PumpAmmInstructionType::Sell => {
                    if instr.accounts.len() < 19 {
                        warn!("PumpAMM Sell: The instruction does not contain a minimum of 17 accounts needed.");
                        continue;
                    }
                    deserialize_pumpamm_sell_instruction(&instr.data, &instr.accounts, account_keys, &is_signer, &is_writable)
                },
                PumpAmmInstructionType::CreatePool => {
                    if instr.accounts.len() < 18 {
                        warn!("PumpAMM CreatePool: The instruction does not contain a minimum of 18 accounts needed.");
                        continue;
                    }
                    deserialize_pumpamm_create_pool_instruction(&instr.data, &instr.accounts, account_keys, &is_signer, &is_writable)
                },
                _ => {
                    continue;
                }
            };

            match decoded_result {
                Ok(decoded) => serialized_instructions.push(decoded),
                Err(err) => {
                    error!("Error decoding PumpAMM instruction: {}", err);
                    continue;
                }
            }
        } else if program_key == &RAYDIUM_CPMM_PROGRAM {
            let Some(instr_type) = get_raydium_cpmm_instruction_type(&instr.data) else { continue };
            contains_relevant_instruction = true;
            
            let decoded_result = match instr_type {
                RaydiumCpmmInstructionType::Initialize => {
                    if instr.accounts.len() < 20 {
                        warn!("Raydium CPMM Initialize: The instruction does not contain a minimum of 20 accounts needed.");
                        continue;
                    }
                    deserialize_raydium_cpmm_initialize_instruction(&instr.data, &instr.accounts, account_keys, &is_signer, &is_writable)
                }
            };
            match decoded_result {
                Ok(decoded) => serialized_instructions.push(decoded),
                Err(err) => {
                    error!("Error decoding Raydium CPMM instruction: {}", err);
                    continue;
                }
            }
        } else if program_key == &METEORA_VCURVE_PROGRAM_ID {
            let Some(instr_type) = get_meteora_vcurve_instruction_type(&instr.data) else { continue };
            contains_relevant_instruction = true;
            
            let decoded_result = match instr_type {
                MeteoraVCurveInstructionType::InitializeVirtualPoolWithSplToken => {
                    if instr.accounts.len() < 16 {
                        warn!("Meteora VCurve Initialize Virtual Pool With SPL Token: The instruction does not contain a minimum of 16 accounts needed.");
                        continue;
                    }
                    deserialize_meteora_vcurve_initialize_virtual_pool_instruction(&instr.data, &instr.accounts, account_keys, &is_signer, &is_writable)
                }
            };
            match decoded_result {
                Ok(decoded) => serialized_instructions.push(decoded),
                Err(err) => {
                    error!("Error decoding Meteora VCurve instruction: {}", err);
                    continue;
                }
            }
        } else if program_key == &METEORADYN_PROGRAM_ID {
            let Some(instr_type) = get_meteoradyn_instruction_type(&instr.data) else { continue };
            contains_relevant_instruction = true;
            
            let decoded_result = match instr_type {
                MeteoraDynInstructionType::InitializePermissionlessPool => {
                    if instr.accounts.len() < 24 {
                        warn!("Meteora DYN InitializePermissionlessPool: The instruction does not contain a minimum of 24 accounts needed.");
                        continue;
                    }
                    deserialize_meteoradyn_initialize_permissionless_pool_instruction(&instr.data, &instr.accounts, account_keys, &is_signer, &is_writable)
                },
                _ => {
                    continue;
                }
            };

            match decoded_result {
                Ok(decoded) => serialized_instructions.push(decoded),
                Err(err) => {
                    error!("Error decoding Meteora DYN instruction: {}", err);
                    continue;
                }
            }
        } else if program_key == &METEORA_AMM_V2_PROGRAM_ID {
            let Some(instr_type) = get_meteora_amm_v2_instruction_type(&instr.data) else { continue };
            contains_relevant_instruction = true;
            
            let decoded_result = match instr_type {
                MeteoraAmmV2InstructionType::CreatePool1 |
                MeteoraAmmV2InstructionType::CreatePool2 |
                MeteoraAmmV2InstructionType::CreatePool3 => {
                    if instr.accounts.len() < 12 {
                        warn!("Meteora AMM V2 CreatePool: The instruction does not contain a minimum of 12 accounts needed.");
                        continue;
                    }
                    deserialize_meteora_amm_v2_create_pool_instruction(instr_type, &instr.data, &instr.accounts, account_keys, &is_signer, &is_writable)
                },
                MeteoraAmmV2InstructionType::Swap => {
                    if instr.accounts.len() < 9 {
                        warn!("Meteora AMM V2 Swap: The instruction does not contain a minimum of 9 accounts needed.");
                        continue;
                    }
                    deserialize_meteora_amm_v2_swap_instruction(&instr.data, &instr.accounts, account_keys, &is_signer, &is_writable)
                },
                MeteoraAmmV2InstructionType::AddLiquidity1 |
                MeteoraAmmV2InstructionType::AddLiquidity2 |
                MeteoraAmmV2InstructionType::AddLiquidity3 |
                MeteoraAmmV2InstructionType::AddLiquidity4 => {
                    if instr.accounts.len() < 13 {
                        warn!("Meteora AMM V2 AddLiquidity: The instruction does not contain a minimum of 13 accounts needed.");
                        continue;
                    }
                    deserialize_meteora_amm_v2_add_liquidity_instruction(instr_type, &instr.data, &instr.accounts, account_keys, &is_signer, &is_writable)
                },
                MeteoraAmmV2InstructionType::RemoveLiquidity1 |
                MeteoraAmmV2InstructionType::RemoveLiquidity2 => {
                    if instr.accounts.len() < 7 {
                        warn!("Meteora AMM V2 RemoveLiquidity: The instruction does not contain a minimum of 7 accounts needed.");
                        continue;
                    }
                    deserialize_meteora_amm_v2_remove_liquidity_instruction(instr_type, &instr.data, &instr.accounts, account_keys, &is_signer, &is_writable)
                },
                _ => {
                    continue;
                }
            };

            match decoded_result {
                Ok(decoded) => serialized_instructions.push(decoded),
                Err(err) => {
                    error!("Error decoding Meteora AMM V2 instruction: {}", err);
                    continue;
                }
            }
        } else if program_key == &ORCA_WHIRLPOOL_PROGRAM_ID {
            let Some(instr_type) = get_orca_instruction_type(&instr.data) else { continue };
            contains_relevant_instruction = true;
            
            let decoded_result = match instr_type {
                OrcaInstructionType::Swap => {
                    if instr.accounts.len() < 11 {
                        warn!("Orca Swap: The instruction does not contain a minimum of 11 accounts needed.");
                        continue;
                    }
                    deserialize_orca_swap_instruction(&instr.data, &instr.accounts, account_keys, &is_signer, &is_writable)
                },
                OrcaInstructionType::SwapV2 => {
                    if instr.accounts.len() < 15 {
                        warn!("Orca SwapV2: The instruction does not contain a minimum of 15 accounts needed.");
                        continue;
                    }
                    deserialize_orca_swap_v2_instruction(&instr.data, &instr.accounts, account_keys, &is_signer, &is_writable)
                },
                OrcaInstructionType::IncreaseLiquidity => {
                    if instr.accounts.len() < 11 {
                        warn!("Orca IncreaseLiquidity: The instruction does not contain a minimum of 11 accounts needed.");
                        continue;
                    }
                    deserialize_orca_increase_liquidity_instruction(&instr.data, &instr.accounts, account_keys, &is_signer, &is_writable)
                },
                OrcaInstructionType::DecreaseLiquidity => {
                    if instr.accounts.len() < 11 {
                        warn!("Orca DecreaseLiquidity: The instruction does not contain a minimum of 11 accounts needed.");
                        continue;
                    }
                    deserialize_orca_decrease_liquidity_instruction(&instr.data, &instr.accounts, account_keys, &is_signer, &is_writable)
                },
                OrcaInstructionType::InitializePool => {
                    if instr.accounts.len() < 11 {
                        warn!("Orca InitializePool: The instruction does not contain a minimum of 11 accounts needed.");
                        continue;
                    }
                    deserialize_orca_initialize_pool_instruction(&instr.data, &instr.accounts, account_keys, &is_signer, &is_writable)
                },
                OrcaInstructionType::OpenPosition => {
                    if instr.accounts.len() < 10 {
                        warn!("Orca OpenPosition: The instruction does not contain a minimum of 10 accounts needed.");
                        continue;
                    }
                    deserialize_orca_open_position_instruction(&instr.data, &instr.accounts, account_keys, &is_signer, &is_writable)
                },
                OrcaInstructionType::ClosePosition => {
                    if instr.accounts.len() < 6 {
                        warn!("Orca ClosePosition: The instruction does not contain a minimum of 6 accounts needed.");
                        continue;
                    }
                    deserialize_orca_close_position_instruction(&instr.data, &instr.accounts, account_keys, &is_signer, &is_writable)
                },
                OrcaInstructionType::TwoHopSwap => {
                    if instr.accounts.len() < 20 {
                        warn!("Orca TwoHopSwap: The instruction does not contain a minimum of 20 accounts needed.");
                        continue;
                    }
                    deserialize_orca_two_hop_swap_instruction(&instr.data, &instr.accounts, account_keys, &is_signer, &is_writable)
                },
            };

            match decoded_result {
                Ok(decoded) => serialized_instructions.push(decoded),
                Err(err) => {
                    error!("Error decoding Orca instruction: {}", err);
                    continue;
                }
            }
        }
    }

    if !contains_relevant_instruction || serialized_instructions.is_empty() {
        return None;
    }

    let signatures: Vec<String> = transaction
        .signatures
        .iter()
        .map(|signature| bs58::encode(signature).into_string())
        .collect();

    Some(object! {
        "signatures" => signatures,
        "slot" => slot,
        "message" => object! {
            "header" => object! {
                "numRequiredSignatures" => header.num_required_signatures,
                "numReadonlySignedAccounts" => header.num_readonly_signed_accounts,
                "numReadonlyUnsignedAccounts" => header.num_readonly_unsigned_accounts,
            },
            "recentBlockhash" => bs58::encode(recent_blockhash).into_string(),
            "instructions" => serialized_instructions,
        }
    })
}