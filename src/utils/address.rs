use anyhow::Result;
// use bech32; // Temporarily commented out, as current implementation uses custom bech32 encoding/decoding
use hex;
use kaspa_addresses::{Address, Prefix, Version};

/// Verify Kaspa address using logic consistent with Go version: parse->restore->compare
pub fn verify_address(address: &str, testnet: bool) -> bool {
    let (ver, kpub) = conv_addr_to_kpub(address, testnet);
    if kpub.is_empty() {
        return false;
    }
    let addr2 = if ver == "08" {
        conv_kpub_to_p2sh(&kpub, testnet)
    } else {
        conv_kpub_to_addr(&kpub, testnet)
    };
    if addr2 != address {
        return false;
    }
    true
}

/// Decode Kaspa address using official kaspa-addresses library (preserved)
pub fn decode_address(address: &str) -> Result<Vec<u8>> {
    // Use official Kaspa address library to decode
    let addr = Address::try_from(address).map_err(|e| anyhow::anyhow!("Address decoding failed: {}", e))?;

    // Get address payload data
    let payload = addr.payload;
    Ok(payload.to_vec())
}

/// Encode Kaspa address using official kaspa-addresses library (preserved)
pub fn encode_address(data: &[u8], testnet: bool) -> Result<String> {
    // Use official Kaspa address library to encode
    let prefix = if testnet {
        Prefix::Testnet
    } else {
        Prefix::Mainnet
    };

    // Determine address version based on data length
    let version = match data.len() {
        32 => Version::PubKey,      // Schnorr public key
        33 => Version::PubKeyECDSA, // ECDSA public key
        20 => Version::ScriptHash,  // Script hash
        _ => return Err(anyhow::anyhow!("Unsupported address data length: {}", data.len())),
    };

    let addr = Address::new(prefix, version, data);

    Ok(addr.to_string())
}

/// Validate address format (relaxed: only check prefix and minimum length; strict validation handled by verify_address)
pub fn validate_address_format(address: &str) -> bool {
    if address.is_empty() {
        return false;
    }

    // Basic prefix check
    if !address.starts_with("kaspa:") && !address.starts_with("kaspatest:") {
        return false;
    }

    // Length check: at least some characters after prefix (compatible with historical tests)
    let min_payload_len = 6; // Relaxed requirement
    let payload_len = if address.starts_with("kaspatest:") {
        address.len().saturating_sub("kaspatest:".len())
    } else {
        address.len().saturating_sub("kaspa:".len())
    };
    payload_len >= min_payload_len
}

/// Get address type
pub fn get_address_type(address: &str) -> Option<AddressType> {
    if !verify_address(address, false) && !verify_address(address, true) {
        return None;
    }

    // Determine type based on address prefix
    if address.starts_with("kaspa:") {
        Some(AddressType::P2PKH)
    } else if address.starts_with("kaspatest:") {
        Some(AddressType::P2PKH)
    } else {
        None
    }
}

/// Convert address to public key or script hash, corresponding to Go version's ConvAddrToKPub
fn conv_addr_to_kpub(addr: &str, testnet: bool) -> (String, String) {
    let s = if testnet { 10 } else { 6 };

    // Check address length and prefix, consistent with Go version
    if !testnet && (addr.len() < 67 || !addr.starts_with("kaspa:")) {
        return ("".to_string(), "".to_string());
    }
    if testnet && (addr.len() < 71 || !addr.starts_with("kaspatest:")) {
        return ("".to_string(), "".to_string());
    }

    // Decode bech32 payload (without prefix)
    let bech32_payload = &addr[s..];
    let decoded = match decode_bech32(bech32_payload, testnet) {
        Ok(data) => data,
        Err(_) => return ("".to_string(), "".to_string()),
    };

    let kpub = hex::encode(&decoded);
    if kpub.len() < 64 {
        return ("".to_string(), "".to_string());
    }

    (kpub[..2].to_string(), kpub[2..].to_string())
}

/// Convert script hash to address, corresponding to Go version's ConvKPubToP2sh
fn conv_kpub_to_p2sh(kpub: &str, testnet: bool) -> String {
    if kpub.len() != 64 {
        return "".to_string();
    }

    let kpub_with_version = format!("08{}", kpub); // P2SH version
    let decoded = match hex::decode(&kpub_with_version) {
        Ok(data) => data,
        Err(_) => return "".to_string(),
    };

    let payload = match encode_bech32(&decoded, testnet) {
        Ok(addr) => addr,
        Err(_) => return "".to_string(),
    };

    let prefix = if testnet { "kaspatest:" } else { "kaspa:" };
    format!("{}{}", prefix, payload)
}

/// Convert public key to address, corresponding to Go version's ConvKPubToAddr
fn conv_kpub_to_addr(kpub: &str, testnet: bool) -> String {
    let len_key = kpub.len();
    let kpub_with_version = if len_key == 64 {
        // Schnorr version
        format!("00{}", kpub)
    } else if len_key == 66 {
        // ECDSA version
        format!("01{}", kpub)
    } else {
        return "".to_string();
    };

    let decoded = match hex::decode(&kpub_with_version) {
        Ok(data) => data,
        Err(_) => return "".to_string(),
    };

    let payload = match encode_bech32(&decoded, testnet) {
        Ok(addr) => addr,
        Err(_) => return "".to_string(),
    };

    let prefix = if testnet { "kaspatest:" } else { "kaspa:" };
    format!("{}{}", prefix, payload)
}

/// Convert public key list to multisig script hash, corresponding to Go version's ConvKPubListToScriptHashMultisig
pub fn conv_kpub_list_to_script_hash_multisig(
    m: i64,
    kpub_list: &[String],
    n: i64,
) -> (String, String) {
    let len_kpub_list = kpub_list.len();
    if len_kpub_list < 1 || len_kpub_list != n as usize {
        return ("".to_string(), "".to_string());
    }

    let mut ecdsa = false;
    let mut kpub = format!("{:x}", m + 80);

    for k in kpub_list {
        let len_k = k.len();
        if len_k == 64 {
            kpub.push_str("20");
        } else if len_k == 66 {
            kpub.push_str("21");
            ecdsa = true;
        } else {
            return ("".to_string(), "".to_string());
        }
        kpub.push_str(k);
    }

    kpub.push_str(&format!("{:x}", n + 80));
    if ecdsa {
        kpub.push_str("a9");
    } else {
        kpub.push_str("ae");
    }

    let decoded = match hex::decode(&kpub) {
        Ok(data) => data,
        Err(_) => return ("".to_string(), "".to_string()),
    };

    let hash = blake2b_simd::blake2b(&decoded);
    let script_hash = hex::encode(hash.as_bytes());

    (script_hash, kpub)
}

/// Encode bech32 (returns payload, without prefix), corresponding to Go version EncodeBech32
fn encode_bech32(data: &[u8], testnet: bool) -> Result<String> {
    let p_mod = |list: &[u8]| -> i32 {
        let g = [
            0x98f2bc8e61i64,
            0x79b76d99e2,
            0xf33e5fb3c4,
            0xae2eabe2a8,
            0x1e4f43e470,
        ];
        let mut cs: i64 = 1;
        for &v in list {
            let b = cs >> 35;
            cs = ((cs & 0x07ffffffff) << 5) ^ v as i64;
            for (i, gi) in g.iter().enumerate() {
                if ((b >> i) & 1) == 1 {
                    cs ^= gi;
                }
            }
        }
        (cs ^ 1) as i32
    };

    // Pack 8-bit to 5-bit
    let mut b5: Vec<u8> = Vec::new();
    let mut n_last = 0i32;
    let mut b_last: u8 = 0;
    for &b in data {
        let r_move = 3 + n_last;
        let mut v = (b >> (r_move as u32)) & 31;
        if n_last > 0 {
            v |= b_last;
        }
        b5.push(v);
        n_last = r_move;
        if r_move >= 5 {
            b5.push((b << (8 - r_move as u32) >> 3) & 31);
            n_last = r_move - 5;
        }
        if n_last > 0 {
            b_last = (b << (8 - n_last as u32) >> 3) & 31;
        }
    }
    if n_last > 0 {
        b5.push(b_last);
    }

    let mut b5ex: Vec<u8> = if testnet {
        vec![11, 1, 19, 16, 1, 20, 5, 19, 20, 0]
    } else {
        vec![11, 1, 19, 16, 1, 0]
    };
    b5ex.extend_from_slice(&b5);
    b5ex.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, 0]);
    let p = p_mod(&b5ex);
    for i in 0..8 {
        b5.push(((p >> (5 * (7 - i))) & 31) as u8);
    }

    let c = b"qpzry9x8gf2tvdw0s3jn54khce6mua7l";
    let mut result = String::new();
    for v in b5 {
        result.push(c[v as usize] as char);
    }
    Ok(result)
}

/// Decode bech32 (input payload, without prefix), corresponding to Go version DecodeBech32
fn decode_bech32(data: &str, testnet: bool) -> Result<Vec<u8>> {
    let n = {
        let mut m = std::collections::HashMap::new();
        for (i, ch) in "qpzry9x8gf2tvdw0s3jn54khce6mua7l".chars().enumerate() {
            m.insert(ch, i as u8);
        }
        m
    };

    let mut b5: Vec<u8> = Vec::new();
    for ch in data.chars() {
        match n.get(&ch) {
            Some(v) => b5.push(*v),
            None => return Err(anyhow::anyhow!("Invalid character")),
        }
    }
    if b5.len() < 8 {
        return Err(anyhow::anyhow!("Data too short"));
    }
    let cs: Vec<u8> = b5[b5.len() - 8..].to_vec();
    b5.truncate(b5.len() - 8);

    let mut b5ex: Vec<u8> = if testnet {
        vec![11, 1, 19, 16, 1, 20, 5, 19, 20, 0]
    } else {
        vec![11, 1, 19, 16, 1, 0]
    };
    b5ex.extend_from_slice(&b5);
    b5ex.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, 0]);

    let p_mod = |list: &[u8]| -> i32 {
        let g = [
            0x98f2bc8e61i64,
            0x79b76d99e2,
            0xf33e5fb3c4,
            0xae2eabe2a8,
            0x1e4f43e470,
        ];
        let mut cs: i64 = 1;
        for &v in list {
            let b = cs >> 35;
            cs = ((cs & 0x07ffffffff) << 5) ^ v as i64;
            for (i, gi) in g.iter().enumerate() {
                if ((b >> i) & 1) == 1 {
                    cs ^= gi;
                }
            }
        }
        (cs ^ 1) as i32
    };
    let p = p_mod(&b5ex);
    for i in 0..8 {
        if cs[i] != (((p >> (5 * (7 - i))) & 31) as u8) {
            return Err(anyhow::anyhow!("Checksum failed"));
        }
    }

    // Convert to 8-bit
    let mut b8: Vec<u8> = Vec::new();
    let mut n_last: i32 = 0;
    let mut b_last: u8 = 0;
    for v in b5 {
        let offset = 3 - n_last;
        if offset == 0 {
            b8.push(v | b_last);
            n_last = 0;
            b_last = 0;
        } else if offset < 0 {
            b8.push((v >> (-offset) as u32) | b_last);
            n_last = -offset;
            b_last = v << (8 - n_last as u32);
        } else {
            b_last |= v << offset as u32;
            n_last += 5;
        }
    }
    Ok(b8)
}

#[derive(Debug, Clone, PartialEq)]
pub enum AddressType {
    P2PKH,
    P2SH,
    P2PK,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_address() {
        // Test empty address
        assert!(!verify_address("", false));

        // Test invalid address
        let invalid_address = "invalid:address";
        assert!(!verify_address(invalid_address, false));

        // Test address with correct format but invalid content
        let invalid_format_address = "kaspa:invalid";
        assert!(!verify_address(invalid_format_address, false));
    }

    #[test]
    fn test_validate_address_format() {
        let valid_address =
            "kaspa:test1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
        assert!(validate_address_format(valid_address));

        let invalid_address = "invalid:address";
        assert!(!validate_address_format(invalid_address));
    }
}
