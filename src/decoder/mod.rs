/// Decoder module for standardized instruction parsing across DeFi protocols.
///
/// This module provides a trait-based system for decoding Solana transaction
/// instructions from various DeFi protocols (PumpAMM, Raydium, Orca, etc.) into
/// a unified JSON format.
///
/// # Architecture
///
/// The decoder system consists of two main traits:
///
/// 1. **`InstructionDecoder`** - Core trait that protocol decoders implement
///    - Defines protocol-specific instruction types
///    - Handles instruction identification via discriminators
///    - Parses instruction parameters
///    - Produces standardized JSON output
///
/// 2. **`TransactionProcessor`** - Higher-level trait for unified processing
///    - Automatically implemented for all `InstructionDecoder` types
///    - Provides single entry point: `process_instruction()`
///    - Handles validation and error handling
///
/// # Example Usage
///
/// ```rust,ignore
/// use crate::decoder::{InstructionDecoder, TransactionProcessor};
/// use crate::decoder::pump_amm::PumpAmmDecoder;
///
/// // In transaction processing loop:
/// if program_key == PumpAmmDecoder::program_id() {
///     if let Some(result) = PumpAmmDecoder::process_instruction(
///         &instr.data,
///         &instr.accounts,
///         account_keys,
///         &is_signer,
///         &is_writable
///     ) {
///         match result {
///             Ok(decoded_json) => {
///                 // Successfully decoded instruction
///                 serialized_instructions.push(decoded_json);
///             }
///             Err(e) => {
///                 error!("Failed to decode PumpAMM instruction: {}", e);
///             }
///         }
///     }
/// }
/// ```
///
/// # Adding New Protocol Decoders
///
/// To add support for a new protocol:
///
/// 1. Create a new file in this module (e.g., `raydium.rs`)
/// 2. Define the instruction type enum
/// 3. Define parameter structs with Borsh serialization
/// 4. Create a decoder struct and implement `InstructionDecoder`
/// 5. Add the module to `mod.rs`
///
/// Example:
///
/// ```rust,ignore
/// pub struct RaydiumDecoder;
///
/// impl InstructionDecoder for RaydiumDecoder {
///     type InstructionType = RaydiumInstructionType;
///
///     fn program_id() -> &'static Pubkey { /* ... */ }
///     fn protocol_name() -> &'static str { "Raydium" }
///     fn identify_instruction(data: &[u8]) -> Option<Self::InstructionType> { /* ... */ }
///     fn decode_instruction(/* ... */) -> Result<JsonValue, String> { /* ... */ }
/// }
/// ```

// Core decoder traits
pub mod decoder;
pub mod byte_parser;
pub mod pump_amm;
pub mod pumpfun;
pub mod boop;
pub mod meteora_damm;
pub mod meteora_dlmm;
pub mod meteora_dbc;
pub mod meteora_vcurve;
pub use decoder::{InstructionDecoder, TransactionProcessor};
pub use pump_amm::PumpAmmDecoder;