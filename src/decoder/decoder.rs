use json::JsonValue;
use solana_sdk::pubkey::Pubkey;

pub trait InstructionDecoder {
    type InstructionType;

    fn program_id() -> &'static Pubkey;
    fn protocol_name() -> &'static str;
    fn identify_instruction(data: &[u8]) -> Option<Self::InstructionType>;
    fn decode_instruction(
        instruction_type: Self::InstructionType,
        data: &[u8],
        accounts_indices: &[u8],
        account_keys: &[Pubkey],
        is_signer: &[bool],
        is_writable: &[bool],
    ) -> Result<JsonValue, String>;

    fn validate_account_count(_instruction_type: &Self::InstructionType, _account_count: usize) -> bool {
        true // default: no validation
    }
}

/// Higher-level trait for processing instructions with a unified interface.
///
/// This trait builds on `InstructionDecoder` to provide a single entry point
/// for processing instructions. It handles common logic like data validation,
/// instruction identification, and error handling.
///
/// This trait is automatically implemented for any type that implements
/// `InstructionDecoder`, providing a default implementation of `process_instruction`.
///
/// # Usage in Transaction Processing
///
/// In the main transaction processing loop, you can use this trait to simplify
/// instruction handling:
///
/// ```rust,ignore
/// if program_key == PumpAmmDecoder::program_id() {
///     if let Some(decoded) = PumpAmmDecoder::process_instruction(
///         &instr.data,
///         &instr.accounts,
///         account_keys,
///         &is_signer,
///         &is_writable
///     ) {
///         match decoded {
///             Ok(json) => serialized_instructions.push(json),
///             Err(e) => error!("Decoding error: {}", e),
///         }
///     }
/// }
/// ```
pub trait TransactionProcessor: InstructionDecoder {
    fn process_instruction(
        data: &[u8],
        accounts_indices: &[u8],
        account_keys: &[Pubkey],
        is_signer: &[bool],
        is_writable: &[bool],
    ) -> Option<Result<JsonValue, String>> {
        if data.len() < 8 {
            return None;
        }

        let instruction_type = Self::identify_instruction(data)?;

        if !Self::validate_account_count(&instruction_type, accounts_indices.len()) {
            return Some(Err(format!(
                "{} instruction has insufficient accounts: got {}",
                Self::protocol_name(),
                accounts_indices.len()
            )));
        }

        // Decode the instruction
        Some(Self::decode_instruction(
            instruction_type,
            data,
            accounts_indices,
            account_keys,
            is_signer,
            is_writable,
        ))
    }
}

impl<T: InstructionDecoder> TransactionProcessor for T {}