use solana_sdk::pubkey::Pubkey;

// u32 for string len; utf8 bytes
pub fn parse_string(data: &[u8], offset: &mut usize) -> Result<String, String> {
    if data.len() < *offset + 4 {
        return Err("Insufficient data for string length".to_string());
    }
    let str_len = u32::from_le_bytes(
        data[*offset..*offset + 4]
            .try_into()
            .map_err(|_| "Failed to parse string length")?
    ) as usize;
    *offset += 4;
    
    if data.len() < *offset + str_len {
        return Err("Insufficient data for string".to_string());
    }
    let string = String::from_utf8(data[*offset..*offset + str_len].to_vec())
        .map_err(|e| format!("Failed to parse string as UTF-8: {:?}", e))?;
    *offset += str_len;

    Ok(string)
}

// u64 8 bytes le
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

// pubkey 32 bytes
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

// bool 1 byte
pub fn parse_bool(data: &[u8], offset: &mut usize) -> Result<bool, String> {
    if data.len() < *offset + 1 {
        return Err("Insufficient data for bool".to_string());
    }
    let value = data[*offset] != 0;
    *offset += 1;
    Ok(value)
}

/// Parse a u32 from bytes (4 bytes, little endian)
pub fn parse_u32(data: &[u8], offset: &mut usize) -> Result<u32, String> {
    if data.len() < *offset + 4 {
        return Err("Insufficient data for u32".to_string());
    }
    let value = u32::from_le_bytes(
        data[*offset..*offset + 4]
            .try_into()
            .map_err(|_| "Failed to parse u32")?
    );
    *offset += 4;
    Ok(value)
}

/// Parse a u16 from bytes (2 bytes, little endian)
pub fn parse_u16(data: &[u8], offset: &mut usize) -> Result<u16, String> {
    if data.len() < *offset + 2 {
        return Err("Insufficient data for u16".to_string());
    }
    let value = u16::from_le_bytes(
        data[*offset..*offset + 2]
            .try_into()
            .map_err(|_| "Failed to parse u16")?
    );
    *offset += 2;
    Ok(value)
}

/// Parse a u8 from bytes (1 byte)
pub fn parse_u8(data: &[u8], offset: &mut usize) -> Result<u8, String> {
    if data.len() < *offset + 1 {
        return Err("Insufficient data for u8".to_string());
    }
    let value = data[*offset];
    *offset += 1;
    Ok(value)
}

/// Parse an i32 from bytes (4 bytes, little endian)
pub fn parse_i32(data: &[u8], offset: &mut usize) -> Result<i32, String> {
    if data.len() < *offset + 4 {
        return Err("Insufficient data for i32".to_string());
    }
    let value = i32::from_le_bytes(
        data[*offset..*offset + 4]
            .try_into()
            .map_err(|_| "Failed to parse i32")?
    );
    *offset += 4;
    Ok(value)
}

/// Parse an Option<u64> from bytes (1 byte discriminant + optional 8 bytes, little endian)
/// Borsh format: 0 = None, 1 = Some(value)
pub fn parse_option_u64(data: &[u8], offset: &mut usize) -> Result<Option<u64>, String> {
    let discriminant = parse_u8(data, offset)?;

    match discriminant {
        0 => Ok(None),
        1 => {
            let value = parse_u64(data, offset)?;
            Ok(Some(value))
        }
        _ => Err(format!("Invalid Option discriminant: {}", discriminant))
    }
}

/// Parse a vector of elements (u32 length + elements)
/// Takes a parser function that knows how to parse individual elements
pub fn parse_vec<T, F>(data: &[u8], offset: &mut usize, element_parser: F) -> Result<Vec<T>, String>
where
    F: Fn(&[u8], &mut usize) -> Result<T, String>,
{
    // Parse vector length (u32 - 4 bytes, little endian)
    let vec_len = parse_u32(data, offset)? as usize;

    // Parse each element
    let mut vec = Vec::with_capacity(vec_len);
    for i in 0..vec_len {
        let element = element_parser(data, offset)
            .map_err(|e| format!("Failed to parse vector element {}: {}", i, e))?;
        vec.push(element);
    }

    Ok(vec)
}

/// Parse a vector of u64 values
pub fn parse_vec_u64(data: &[u8], offset: &mut usize) -> Result<Vec<u64>, String> {
    parse_vec(data, offset, parse_u64)
}

/// Parse a vector of u32 values
pub fn parse_vec_u32(data: &[u8], offset: &mut usize) -> Result<Vec<u32>, String> {
    parse_vec(data, offset, parse_u32)
}

/// Parse a vector of u16 values
pub fn parse_vec_u16(data: &[u8], offset: &mut usize) -> Result<Vec<u16>, String> {
    parse_vec(data, offset, parse_u16)
}

/// Parse a vector of u8 values
pub fn parse_vec_u8(data: &[u8], offset: &mut usize) -> Result<Vec<u8>, String> {
    parse_vec(data, offset, parse_u8)
}

/// Parse a vector of strings
pub fn parse_vec_string(data: &[u8], offset: &mut usize) -> Result<Vec<String>, String> {
    parse_vec(data, offset, parse_string)
}

/// Parse a vector of Pubkeys
pub fn parse_vec_pubkey(data: &[u8], offset: &mut usize) -> Result<Vec<Pubkey>, String> {
    parse_vec(data, offset, parse_pubkey)
}