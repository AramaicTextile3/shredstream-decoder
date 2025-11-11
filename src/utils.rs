use dashmap::DashMap;
use dotenv::var;
use lazy_static::lazy_static;
use libc;
use std::fs::OpenOptions;
use std::io::{Write, BufWriter};
use serde_json::Value;
use std::collections::VecDeque;
use std::os::unix::io::AsRawFd;
use tokio::net::UdpSocket;
use tracing::{warn, error, debug};
use json::{object, JsonValue};
use solana_sdk::pubkey::Pubkey;

lazy_static! {
    static ref SLOT_STATS: DashMap<u64, (u64, u64, u64, u64)> = DashMap::new();
}

pub async fn create_udp_socket_with_buffer(addr: &str, buffer_size: usize) -> UdpSocket {
    let socket = std::net::UdpSocket::bind(addr).expect("Failed to bind socket");
    socket
        .set_nonblocking(true)
        .expect("Failed to set non-blocking");

    let fd = socket.as_raw_fd();
    unsafe {
        if libc::setsockopt(
            fd,
            libc::SOL_SOCKET,
            libc::SO_RCVBUF,
            &buffer_size as *const _ as *const libc::c_void,
            std::mem::size_of_val(&buffer_size) as libc::socklen_t,
        ) != 0
        {
            panic!("Failed to set UDP receive buffer size");
        }
    }

    UdpSocket::from_std(socket).expect("Failed to convert to Tokio UdpSocket")
}

pub fn increment_slot_counters(
    slot: u64, 
    fec_blocks_count: u64, 
    tx_count: u64, 
    fec_blocks_complete: u64, 
    fec_blocks_incomplete: u64,
) {
    let mut entry = SLOT_STATS.entry(slot).or_insert((0, 0, 0, 0)); 
    entry.value_mut().0 += fec_blocks_count;  
    entry.value_mut().1 += tx_count;          
    entry.value_mut().2 += fec_blocks_complete; 
    entry.value_mut().3 += fec_blocks_incomplete;
    
    debug!("Incrementing counters for slot {}: +{} blocks, +{} txs, +{} complete, +{} incomplete", 
           slot, fec_blocks_count, tx_count, fec_blocks_complete, fec_blocks_incomplete);
}


pub fn save_slot_statistics_to_file(slot: u64) {
    if let Some((fec_blocks_count, tx_count, fec_blocks_complete, fec_blocks_incomplete)) =
        SLOT_STATS.get(&slot).map(|entry| *entry)
    {
        let file_path = "slot_stats.json";

        let stats_data = std::fs::read_to_string(file_path)
            .ok()
            .and_then(|content| serde_json::from_str::<VecDeque<Value>>(&content).ok())
            .unwrap_or_else(|| VecDeque::with_capacity(100));

        let mut stats_vec = stats_data;
        
        if let Some(existing_index) = stats_vec.iter().position(|entry| entry["slot"] == slot) {
            stats_vec.remove(existing_index);
        }
        
        if stats_vec.len() >= 500 {
            stats_vec.pop_front();
        }
        
        stats_vec.push_back(serde_json::json!({
            "slot": slot,
            "fec_blocks_count": fec_blocks_count,
            "tx_count": tx_count,
            "fec_blocks_complete": fec_blocks_complete,
            "fec_blocks_incomplete": fec_blocks_incomplete
        }));
        
        debug!(
            "Saved final statistics for slot {}: fec_blocks_count={}, tx_count={}, fec_blocks_complete={}, fec_blocks_incomplete={}",
            slot, fec_blocks_count, tx_count, fec_blocks_complete, fec_blocks_incomplete
        );

        match OpenOptions::new().write(true).create(true).truncate(true).open(file_path) {
            Ok(file) => {
                let mut writer = BufWriter::new(file);
                if let Err(e) = serde_json::to_writer_pretty(&mut writer, &stats_vec) {
                    error!("Failed to save statistics: {}", e);
                }
                let _ = writer.flush();
            }
            Err(e) => {
                error!("Failed to open file for writing statistics: {}", e);
            }
        }
    } else {
        warn!("No statistics found for slot {}. Skipping save.", slot);
    }
}

pub fn env(key: &str) -> String {
    let value = var(key).unwrap_or_else(|_| {
        panic!("Environment variable '{}' is required but not set in the env file !", key);
    });
    if value.trim().is_empty() {
        panic!("Environment variable '{}' must have a valid value in the env file !", key);
    }

    value
}

pub fn create_standardized_instruction(
    program_id: &Pubkey,
    instruction_name: &str,
    protocol_name: &str,
    raw_data: &[u8],
    accounts_indices: &[u8],
    account_keys: &[Pubkey],
    is_signer: &[bool],
    is_writable: &[bool],
    parsed_args: JsonValue
) -> JsonValue {
    let mapped_accounts: Vec<JsonValue> = accounts_indices.iter()
        .enumerate()
        .map(|(i, &idx)| {
            let account_idx = idx as usize;
            if account_idx < account_keys.len() {
                object! {
                    "index": i,
                    "pubkey": bs58::encode(&account_keys[account_idx]).into_string(),
                    "signer": if account_idx < is_signer.len() { is_signer[account_idx] } else { false },
                    "writable": if account_idx < is_writable.len() { is_writable[account_idx] } else { false }
                }
            } else {
                object! {
                    "index": i,
                    "pubkey": "unknown",
                    "signer": false,
                    "writable": false
                }
            }
        })
        .collect();
    
    let raw_data_hex = raw_data.iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();
    
    object! {
        "program_id": bs58::encode(program_id).into_string(),
        "instruction_name": instruction_name,
        "protocol": protocol_name,
        "raw_data": raw_data_hex,
        "accounts": mapped_accounts,
        "parsed_data": parsed_args
    }
}
