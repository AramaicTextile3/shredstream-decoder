use solana_sdk::pubkey::Pubkey;

/// Parse a length-prefixed string from bytes (u32 length + utf8 bytes)
pub fn parse_string(data: &[u8], offset: &mut usize) -> Result<String, String> {
    // Parse length (u32 - 4 bytes, little endian)
    if data.len() < *offset + 4 {
        return Err("Insufficient data for string length".to_string());
    }
    let str_len = u32::from_le_bytes(
        data[*offset..*offset + 4]
            .try_into()
            .map_err(|_| "Failed to parse string length")?
    ) as usize;
    *offset += 4;

    // Parse string bytes
    if data.len() < *offset + str_len {
        return Err("Insufficient data for string".to_string());
    }
    let string = String::from_utf8(data[*offset..*offset + str_len].to_vec())
        .map_err(|e| format!("Failed to parse string as UTF-8: {:?}", e))?;
    *offset += str_len;

    Ok(string)
}

/// Parse a u64 from bytes (8 bytes, little endian)
pub fn parse_u64(data: &[u8], offset: &mut usize) -> Result<u64, String> {
    if data.len() < *offset + 8 {
        return Err("Insufficient data for u64".to_string());
    }
    let value = u64::from_le_bytes(
        data[*offset..*offset + 8]
            .try_into()
            .map_err(|_| "Failed to parse u64")?
    );
    *offset += 8;
    Ok(value)
}

/// Parse a Pubkey from bytes (32 bytes)
pub fn parse_pubkey(data: &[u8], offset: &mut usize) -> Result<Pubkey, String> {
    if data.len() < *offset + 32 {
        return Err("Insufficient data for pubkey".to_string());
    }
    let pubkey = Pubkey::new_from_array(
        data[*offset..*offset + 32]
            .try_into()
            .map_err(|_| "Failed to parse pubkey")?
    );
    *offset += 32;
    Ok(pubkey)
}

/// Parse a bool from bytes (1 byte)
pub fn parse_bool(data: &[u8], offset: &mut usize) -> Result<bool, String> {
    if data.len() < *offset + 1 {
        return Err("Insufficient data for bool".to_string());
    }
    let value = data[*offset] != 0;
    *offset += 1;
    Ok(value)
}