mod utils;
mod address_lookup_table_cache;
mod decoder;
mod pumpfun_decoding;
mod raydium_decoding;
mod moonit_decoding;
mod raydium_launchlab_decoding;
mod raydium_cpmm_decoding;
mod pumpamm_decoding;
mod meteora_vcurve_decoding;
mod boop_decoding;
mod meteoradyn_decoding;
mod meteora_amm_v2_decoding;
mod shreds_processing;

mod orca_decoding;
mod grpc_server;

use crate::utils::*;
use crate::shreds_processing::*;

use crate::address_lookup_table_cache::*;
use crate::grpc_server::*;

use dashmap::{DashMap, DashSet};
use dotenv::dotenv;
use rayon::ThreadPoolBuilder;
use tokio::sync::{broadcast};
use std::time::Duration;
use std::sync::Arc;
use std::time::{Instant};
use std::fs;
use std::path::Path;
// For logging with tracing
use tracing::{info, warn, error, debug};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, fmt, EnvFilter};
////////////////////////////////////////////////////////////////////////////////
use mimalloc::MiMalloc;
 

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filter_layer = EnvFilter::from_default_env();
    let is_debug = std::env::var("RUST_LOG").map(|lvl| lvl.to_lowercase() == "debug").unwrap_or(false);

    let fmt_layer = fmt::layer()
        .with_thread_ids(is_debug)    
        .with_thread_names(is_debug)  
        .with_line_number(is_debug)   
        .with_target(is_debug);      

    tracing_subscriber::registry()
        .with(filter_layer) 
        .with(fmt_layer)
        .init();
    
    dotenv().ok();

    // Initialize Address Lookup Table Cache
    let rpc_endpoint = std::env::var("RPC_ENDPOINT")
        .unwrap_or_else(|_| "https://mainnet.helius-rpc.com/?api-key=50b3223c-e0fb-4eaa-8923-b11042258828".to_string());
    let cache_ttl_seconds = std::env::var("LOOKUP_TABLE_CACHE_TTL_SECONDS")
        .unwrap_or_else(|_| "300".to_string())
        .parse()
        .unwrap_or(300);
    
    init_lookup_table_cache(&rpc_endpoint, cache_ttl_seconds);
    info!("Initialized Address Lookup Table cache with endpoint: {}", rpc_endpoint);
    
    // Test if cache was initialized properly
    if get_lookup_table_cache().is_some() {
        info!("✅ Address Lookup Table cache is ready");
    } else {
        warn!("❌ Address Lookup Table cache failed to initialize - transactions with lookup tables will show 'Unknown' accounts");
    }

    ThreadPoolBuilder::new()
        .num_threads(num_cpus::get_physical() - 1) // Reserve 1 thread for other processes
        .build_global()
        .expect("Failed to build Rayon thread pool");

    // DashMap structure to store FecBlocks
    let fec_blocks = Arc::new(DashMap::<(u64, u32), FecBlock>::new());
    let processed_blocks = Arc::new(DashSet::new());
    
    // Socket with buffer to receive shreds
    let udp_address = env("UDP_BUFFER_SOCKET");
    let socket: Arc<tokio::net::UdpSocket> = Arc::new(create_udp_socket_with_buffer(&udp_address, 256 * 1024).await);

    // Broadcast channel for transactions
    let (broadcast_tx, _) = broadcast::channel::<(String, u64)>(1000);



    // gRPC Server
    let broadcast_tx_clone = broadcast_tx.clone();
    tokio::spawn(async move {
        let grpc_address = env("GRPC_SERVER_ENDPOINT");
        let grpc_addr: std::net::SocketAddr = grpc_address.parse().expect("Invalid gRPC address format");
        if let Err(e) = serve_grpc(grpc_addr, broadcast_tx_clone).await {
            error!("gRPC server failed: {:?}", e);
        }
    });

    // Task for receiving shreds from Proxy
    let socket_task = {
        let fec_blocks_clone = Arc::clone(&fec_blocks);
        let processed_blocks_gc: Arc<DashSet<(u64, u32)>> = Arc::clone(&processed_blocks);
        let broadcast_tx_clone = broadcast_tx.clone();

        tokio::spawn(async move {
            info!("Shredstream Decoder started ! Starting to listen for shred packets...");
            let mut buf = [0u8; 1232];

            loop {
                debug!("socket_task: Top of recv_from loop, about to call socket.recv_from().await");
                match socket.recv_from(&mut buf).await {
                    Ok((size, _)) => {
                        if size > buf.len() {
                            warn!("Received data size {} exceeds buffer length {}", size, buf.len());
                            continue;
                        }
                        debug!("socket_task: Successfully received {} bytes.", size);
                        let shred_data = Box::from(&buf[..size]);

                        let fec_blocks_clone_inner = Arc::clone(&fec_blocks_clone);
                        let processed_blocks_clone_inner = Arc::clone(&processed_blocks_gc);
                        let broadcast_tx_clone_inner = broadcast_tx_clone.clone();

                        if let Err(e) = async {
                            if let Err(e) = collect_shred(
                                &shred_data,
                                &fec_blocks_clone_inner,
                                &processed_blocks_clone_inner,
                                broadcast_tx_clone_inner,
                            ).await
                            {
                                error!("CollectShredError: {:?}", e);
                            }

                            Ok::<(), ()>(())
                        }
                        .await
                        {
                            error!("Task in socket_task failed: {:?}", e);
                        }
                    }
                    Err(e) => {
                        error!("Error receiving shred: {:?}", e);
                    }
                }
            }
        })
    };

    // Garbage collector eraser
    let fec_blocks_gc: Arc<DashMap<(u64, u32), FecBlock>> = Arc::clone(&fec_blocks);
    let processed_blocks_gc: Arc<DashSet<(u64, u32)>> = Arc::clone(&processed_blocks);
    
    // Task for periodically deleting the slot_stats.json file
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(30)).await;
            
            let file_path = "slot_stats.json";
            
            if Path::new(file_path).exists() {
                match fs::File::create(file_path) {
                    Ok(_) => {
                        info!("The slot_stats.json file was successfully cleaned.");
                        if let Err(e) = fs::write(file_path, "[]") {
                            error!("Error initializing the slot_stats.json file: {}", e);
                        }
                    },
                    Err(e) => {
                        error!("Error cleaning the slot_stats.json file: {}", e);
                    }
                }
            }
        }
    });

    // Task for periodically cleaning the fec_blocks and processed_blocks
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(10)).await;
            
            let before_fec_blocks = fec_blocks_gc.len();
            let before_processed_blocks = processed_blocks_gc.len();
            
            let now = Instant::now();
    
            fec_blocks_gc.retain(|_, fec_block| {
                now.duration_since(fec_block.collection_start.expect("COLLECTION_START not initialized")) < Duration::from_secs(20)
            });
    
            processed_blocks_gc.retain(|&(_, timestamp)| {
                let block_time = Instant::now() - Duration::from_secs(20);
                u64::from(timestamp) > block_time.elapsed().as_secs()
            });
    
            let removed_fec = before_fec_blocks - fec_blocks_gc.len();
            let removed_processed = before_processed_blocks - processed_blocks_gc.len();

            info!(
                "Garbage collector: Removed fec_blocks = {}, Removed processed_blocks = {}",
                removed_fec,
                removed_processed
            );
        }
    });

    tokio::try_join!(
        async {
            socket_task.await.map_err(|e| {
                error!("socket_task failed: {:?}", e);
                Box::<dyn std::error::Error>::from(e)
            })
        },
    )?;
        
    Ok(())
}