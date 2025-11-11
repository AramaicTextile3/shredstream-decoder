use dashmap::DashMap;
use once_cell::sync::Lazy;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    address_lookup_table::state::AddressLookupTable,
    message::v0::MessageAddressTableLookup,
    pubkey::Pubkey,
    instruction::CompiledInstruction,
    transaction::VersionedTransaction,
    message::VersionedMessage,
};
use std::sync::Arc;
use tokio::time::{Duration, Instant};
use tracing::{debug, error, info, warn};

/// Cache entry for a lookup table
#[derive(Clone, Debug)]
pub struct LookupTableCacheEntry {
    pub addresses: Vec<Pubkey>,
    pub last_updated: Instant,
    pub is_valid: bool,
}

/// Address Lookup Table Cache Manager
pub struct AddressLookupTableCache {
    cache: Arc<DashMap<Pubkey, LookupTableCacheEntry>>,
    rpc_client: Arc<RpcClient>,
    cache_ttl: Duration,
}

impl AddressLookupTableCache {
    /// Create a new lookup table cache
    pub fn new(rpc_endpoint: &str, cache_ttl_seconds: u64) -> Self {
        let rpc_client = Arc::new(RpcClient::new(rpc_endpoint.to_string()));
        let cache_ttl = Duration::from_secs(cache_ttl_seconds);
        
        Self {
            cache: Arc::new(DashMap::new()),
            rpc_client,
            cache_ttl,
        }
    }

    /// Get addresses from lookup table, using cache first, then RPC
    pub async fn get_lookup_table_addresses(
        &self,
        lookup_table_pubkey: &Pubkey,
    ) -> Result<Vec<Pubkey>, String> {
        // Check cache first
        if let Some(entry) = self.cache.get(lookup_table_pubkey) {
            if entry.is_valid && entry.last_updated.elapsed() < self.cache_ttl {
                debug!("Lookup table cache hit for {}", lookup_table_pubkey);
                return Ok(entry.addresses.clone());
            }
        }

        // Cache miss or expired, fetch from RPC
        debug!("Lookup table cache miss for {}, fetching via RPC", lookup_table_pubkey);
        match self.fetch_lookup_table_from_rpc(lookup_table_pubkey).await {
            Ok(addresses) => {
                // Update cache
                let entry = LookupTableCacheEntry {
                    addresses: addresses.clone(),
                    last_updated: Instant::now(),
                    is_valid: true,
                };
                self.cache.insert(*lookup_table_pubkey, entry);
                info!("Cached lookup table {} with {} addresses", lookup_table_pubkey, addresses.len());
                Ok(addresses)
            }
            Err(e) => {
                error!("Failed to fetch lookup table {}: {}", lookup_table_pubkey, e);
                
                // If we have a stale cache entry, use it as fallback
                if let Some(entry) = self.cache.get(lookup_table_pubkey) {
                    if entry.is_valid {
                        warn!("Using stale cache entry for lookup table {}", lookup_table_pubkey);
                        return Ok(entry.addresses.clone());
                    }
                }
                
                // Cache the failure to avoid repeated RPC calls
                let entry = LookupTableCacheEntry {
                    addresses: vec![],
                    last_updated: Instant::now(),
                    is_valid: false,
                };
                self.cache.insert(*lookup_table_pubkey, entry);
                
                Err(e)
            }
        }
    }

    /// Force refresh a lookup table from RPC, bypassing cache TTL
    pub async fn force_refresh_lookup_table(
        &self,
        lookup_table_pubkey: &Pubkey,
    ) -> Result<Vec<Pubkey>, String> {
        debug!("Force refreshing lookup table {}", lookup_table_pubkey);
        match self.fetch_lookup_table_from_rpc(lookup_table_pubkey).await {
            Ok(addresses) => {
                let entry = LookupTableCacheEntry {
                    addresses: addresses.clone(),
                    last_updated: Instant::now(),
                    is_valid: true,
                };
                self.cache.insert(*lookup_table_pubkey, entry);
                info!("Force refreshed lookup table {} with {} addresses", lookup_table_pubkey, addresses.len());
                Ok(addresses)
            }
            Err(e) => {
                error!("Failed to force refresh lookup table {}: {}", lookup_table_pubkey, e);
                Err(e)
            }
        }
    }

    /// Fetch lookup table from RPC
    async fn fetch_lookup_table_from_rpc(
        &self,
        lookup_table_pubkey: &Pubkey,
    ) -> Result<Vec<Pubkey>, String> {
        let account_data = self
            .rpc_client
            .get_account_data(lookup_table_pubkey)
            .map_err(|e| format!("RPC error getting lookup table account: {}", e))?;

        // Parse the lookup table account
        let lookup_table = AddressLookupTable::deserialize(&account_data)
            .map_err(|e| format!("Failed to deserialize lookup table: {}", e))?;

        Ok(lookup_table.addresses.to_vec())
    }

    /// Resolve lookup table addresses for transaction account keys
    pub async fn resolve_address_lookups(
        &self,
        base_account_keys: &[Pubkey],
        lookups: &[MessageAddressTableLookup],
    ) -> Vec<Pubkey> {
        let mut resolved_keys = base_account_keys.to_vec();
        debug!("Resolving {} lookup tables for {} base accounts", lookups.len(), base_account_keys.len());

        for lookup in lookups {
            match self.get_lookup_table_addresses(&lookup.account_key).await {
                Ok(mut lookup_addresses) => {
                    let mut needs_refresh = false;
                    
                    // Check if any indices are out of bounds
                    for &index in lookup.readonly_indexes.iter().chain(lookup.writable_indexes.iter()) {
                        if index as usize >= lookup_addresses.len() {
                            needs_refresh = true;
                            break;
                        }
                    }
                    
                    // If out of bounds detected, try to refresh the lookup table
                    if needs_refresh {
                        warn!(
                            "Out of bounds indices detected for lookup table {} (current size: {}), attempting to refresh",
                            lookup.account_key,
                            lookup_addresses.len()
                        );
                        
                        match self.force_refresh_lookup_table(&lookup.account_key).await {
                            Ok(refreshed_addresses) => {
                                info!(
                                    "Successfully refreshed lookup table {} - size changed from {} to {}",
                                    lookup.account_key,
                                    lookup_addresses.len(),
                                    refreshed_addresses.len()
                                );
                                lookup_addresses = refreshed_addresses;
                            }
                            Err(e) => {
                                error!(
                                    "Failed to refresh lookup table {} after out of bounds access: {}",
                                    lookup.account_key, e
                                );
                            }
                        }
                    }
                    
                    // Add readonly addresses
                    for &index in &lookup.readonly_indexes {
                        if let Some(address) = lookup_addresses.get(index as usize) {
                            resolved_keys.push(*address);
                        } else {
                            warn!(
                                "Readonly index {} still out of bounds for lookup table {} (size: {}) after refresh",
                                index,
                                lookup.account_key,
                                lookup_addresses.len()
                            );
                            resolved_keys.push(Pubkey::default()); // placeholder
                        }
                    }

                    // Add writable addresses
                    for &index in &lookup.writable_indexes {
                        if let Some(address) = lookup_addresses.get(index as usize) {
                            resolved_keys.push(*address);
                        } else {
                            warn!(
                                "Writable index {} still out of bounds for lookup table {} (size: {}) after refresh",
                                index,
                                lookup.account_key,
                                lookup_addresses.len()
                            );
                            resolved_keys.push(Pubkey::default()); // placeholder
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to resolve lookup table {}: {}", lookup.account_key, e);
                    // Add placeholder addresses for the failed lookup
                    let total_indices = lookup.readonly_indexes.len() + lookup.writable_indexes.len();
                    for _ in 0..total_indices {
                        resolved_keys.push(Pubkey::default()); // placeholder
                    }
                }
            }
        }

        resolved_keys
    }

    /// Get cache statistics
    pub fn get_cache_stats(&self) -> (usize, usize) {
        let total_entries = self.cache.len();
        let valid_entries = self.cache.iter().filter(|entry| entry.is_valid).count();
        (total_entries, valid_entries)
    }

    /// Clear expired cache entries
    pub fn cleanup_expired_entries(&self) {
        let mut expired_keys = Vec::new();
        
        for entry in self.cache.iter() {
            if entry.last_updated.elapsed() > self.cache_ttl {
                expired_keys.push(*entry.key());
            }
        }

        for key in expired_keys {
            self.cache.remove(&key);
        }
    }

    /// Invalidate cache entry for a specific lookup table to force refresh
    pub fn invalidate_cache_entry(&self, lookup_table_pubkey: &Pubkey) {
        if self.cache.remove(lookup_table_pubkey).is_some() {
            debug!("Invalidated cache entry for lookup table {}", lookup_table_pubkey);
        }
    }

    /// Process a transaction to detect lookup table extensions and invalidate cache
    pub fn process_transaction_for_extensions(&self, transaction: &VersionedTransaction) {
        let instructions = match &transaction.message {
            VersionedMessage::Legacy(msg) => &msg.instructions,
            VersionedMessage::V0(msg) => &msg.instructions,
        };

        for instruction in instructions {
            if self.is_extend_lookup_table_instruction(instruction, &transaction.message) {
                // Get the lookup table account from the instruction
                if let Some(lookup_table_pubkey) = self.extract_lookup_table_from_instruction(instruction, &transaction.message) {
                    info!("Detected ExtendLookupTable instruction for {}, invalidating cache", lookup_table_pubkey);
                    self.invalidate_cache_entry(&lookup_table_pubkey);
                }
            }
        }
    }

    /// Check if an instruction is an ExtendLookupTable instruction
    fn is_extend_lookup_table_instruction(&self, instruction: &CompiledInstruction, message: &VersionedMessage) -> bool {
        // Get the program account
        let account_keys = match message {
            VersionedMessage::Legacy(msg) => &msg.account_keys,
            VersionedMessage::V0(msg) => &msg.account_keys,
        };

        if let Some(program_pubkey) = account_keys.get(instruction.program_id_index as usize) {
            // Check if this is the Address Lookup Table program
            let address_lookup_table_program_id = "AddressLookupTab1e1111111111111111111111111";
            if program_pubkey.to_string() == address_lookup_table_program_id {
                // Check the instruction discriminant for ExtendLookupTable (discriminant 2)
                if instruction.data.len() >= 4 {
                    let discriminant = u32::from_le_bytes([
                        instruction.data[0],
                        instruction.data[1], 
                        instruction.data[2],
                        instruction.data[3],
                    ]);
                    return discriminant == 2; // ExtendLookupTable discriminant
                }
            }
        }
        false
    }

    /// Extract the lookup table pubkey from an ExtendLookupTable instruction
    fn extract_lookup_table_from_instruction(&self, instruction: &CompiledInstruction, message: &VersionedMessage) -> Option<Pubkey> {
        let account_keys = match message {
            VersionedMessage::Legacy(msg) => &msg.account_keys,
            VersionedMessage::V0(msg) => &msg.account_keys,
        };

        // For ExtendLookupTable, the first account is the lookup table
        if !instruction.accounts.is_empty() {
            let lookup_table_index = instruction.accounts[0] as usize;
            account_keys.get(lookup_table_index).copied()
        } else {
            None
        }
    }
}

/// Global lookup table cache instance
static LOOKUP_TABLE_CACHE: Lazy<Option<AddressLookupTableCache>> = Lazy::new(|| {
    std::env::var("RPC_ENDPOINT")
        .ok()
        .map(|endpoint| {
            let cache_ttl = std::env::var("LOOKUP_TABLE_CACHE_TTL_SECONDS")
                .unwrap_or_else(|_| "300".to_string()) // 5 minutes default
                .parse()
                .unwrap_or(300);
            
            AddressLookupTableCache::new(&endpoint, cache_ttl)
        })
});

/// Initialize the global lookup table cache
pub fn init_lookup_table_cache(rpc_endpoint: &str, cache_ttl_seconds: u64) {
    unsafe {
        std::env::set_var("RPC_ENDPOINT", rpc_endpoint);
        std::env::set_var("LOOKUP_TABLE_CACHE_TTL_SECONDS", cache_ttl_seconds.to_string());
    }
    
    // Force initialization of the lazy static
    Lazy::force(&LOOKUP_TABLE_CACHE);
    
    info!("Initialized Address Lookup Table cache with RPC endpoint: {}", rpc_endpoint);
}

/// Get the global lookup table cache instance
pub fn get_lookup_table_cache() -> Option<&'static AddressLookupTableCache> {
    LOOKUP_TABLE_CACHE.as_ref()
}

/// Resolve account keys for a transaction including lookup table addresses
pub async fn resolve_transaction_account_keys(
    base_account_keys: &[Pubkey],
    lookups: Option<&[MessageAddressTableLookup]>,
) -> Vec<Pubkey> {
    match (get_lookup_table_cache(), lookups) {
        (Some(cache), Some(lookups)) if !lookups.is_empty() => {
            debug!("Using lookup table cache to resolve {} lookups", lookups.len());
            cache.resolve_address_lookups(base_account_keys, lookups).await
        }
        (None, Some(lookups)) if !lookups.is_empty() => {
            warn!("Lookup table cache not available, but {} lookups needed - falling back to base keys", lookups.len());
            base_account_keys.to_vec()
        }
        _ => {
            // No cache available or no lookups needed
            debug!("No lookup tables needed, using {} base account keys", base_account_keys.len());
            base_account_keys.to_vec()
        }
    }
}

/// Process a transaction to detect and handle lookup table extensions
pub fn process_transaction_for_lookup_table_extensions(transaction: &VersionedTransaction) {
    if let Some(cache) = get_lookup_table_cache() {
        cache.process_transaction_for_extensions(transaction);
    }
}