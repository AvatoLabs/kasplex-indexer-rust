use anyhow::Result;
use hex;

/// KRC-20 script builder
/// Ensure Go and Rust versions generate identical scripts
pub struct ScriptBuilder;

impl ScriptBuilder {
    /// Build issue script
    /// Field order: p, op, tick, name, max, dec, desc
    pub fn build_issue_script(
        tick: &str,
        name: &str,
        max_supply: u64,
        decimals: u8,
        description: &str,
    ) -> Result<String> {
        // Use fixed key order manual assembly with serde_json::to_string for value escaping to ensure order and escaping are completely consistent
        let tick_escaped = serde_json::to_string(tick)?; // "..."
        let name_escaped = serde_json::to_string(name)?; // "..."
        let desc_escaped = serde_json::to_string(description)?; // "..."
        let json_str = format!(
            "{{\"p\":\"KRC-20\",\"op\":\"issue\",\"tick\":{},\"name\":{},\"max\":\"{}\",\"dec\":\"{}\",\"desc\":{}}}",
            tick_escaped, name_escaped, max_supply, decimals, desc_escaped
        );
        Ok(hex::encode(json_str.as_bytes()))
    }

    /// Build transfer script
    /// Field order: p, op, tick, to, amt
    pub fn build_send_script(tick: &str, to_address: &str, amount: &str) -> Result<String> {
        let json_str = format!(
            r#"{{"p":"KRC-20","op":"send","tick":"{}","to":"{}","amt":"{}"}}"#,
            tick, to_address, amount
        );

        Ok(hex::encode(json_str.as_bytes()))
    }

    /// Build mint script
    /// Field order: p, op, tick, to, amt
    pub fn build_mint_script(tick: &str, to_address: &str, amount: &str) -> Result<String> {
        let json_str = format!(
            r#"{{"p":"KRC-20","op":"mint","tick":"{}","to":"{}","amt":"{}"}}"#,
            tick, to_address, amount
        );

        Ok(hex::encode(json_str.as_bytes()))
    }

    /// Build deploy script
    /// Field order: p, op, tick, name, max, dec, desc
    pub fn build_deploy_script(
        tick: &str,
        name: &str,
        max_supply: u64,
        decimals: u8,
        description: &str,
    ) -> Result<String> {
        // Use fixed key order manual assembly with serde_json::to_string for value escaping
        let tick_escaped = serde_json::to_string(tick)?; // "..."
        let name_escaped = serde_json::to_string(name)?; // "..."
        let desc_escaped = serde_json::to_string(description)?; // "..."
        let json_str = format!(
            "{{\"p\":\"KRC-20\",\"op\":\"deploy\",\"tick\":{},\"name\":{},\"max\":\"{}\",\"dec\":\"{}\",\"desc\":{}}}",
            tick_escaped, name_escaped, max_supply, decimals, desc_escaped
        );
        Ok(hex::encode(json_str.as_bytes()))
    }

    /// Build burn script
    /// Field order: p, op, tick, amt
    pub fn build_burn_script(tick: &str, amount: &str) -> Result<String> {
        let json_str = format!(
            r#"{{"p":"KRC-20","op":"burn","tick":"{}","amt":"{}"}}"#,
            tick, amount
        );

        Ok(hex::encode(json_str.as_bytes()))
    }

    /// Build transfer ownership script
    /// Field order: p, op, tick, to
    pub fn build_chown_script(tick: &str, to_address: &str) -> Result<String> {
        let json_str = format!(
            r#"{{"p":"KRC-20","op":"chown","tick":"{}","to":"{}"}}"#,
            tick, to_address
        );

        Ok(hex::encode(json_str.as_bytes()))
    }

    /// Build blacklist script
    /// Field order: p, op, tick, blacklist
    pub fn build_blacklist_script(tick: &str, blacklist: &str) -> Result<String> {
        let json_str = format!(
            r#"{{"p":"KRC-20","op":"blacklist","tick":"{}","blacklist":"{}"}}"#,
            tick, blacklist
        );

        Ok(hex::encode(json_str.as_bytes()))
    }

    /// Build list script
    /// Field order: p, op, tick, list
    pub fn build_list_script(tick: &str, list: &str) -> Result<String> {
        let json_str = format!(
            r#"{{"p":"KRC-20","op":"list","tick":"{}","list":"{}"}}"#,
            tick, list
        );

        Ok(hex::encode(json_str.as_bytes()))
    }

    /// Validate script format
    pub fn validate_script(script_hex: &str) -> Result<bool> {
        // Decode hexadecimal
        let decoded_bytes = hex::decode(script_hex)?;
        let json_str = String::from_utf8(decoded_bytes)?;

        // Parse JSON
        let json_value: serde_json::Value = serde_json::from_str(&json_str)?;

        // Validate required fields
        if let Some(obj) = json_value.as_object() {
            // Check protocol field
            if obj.get("p").and_then(|v| v.as_str()) != Some("KRC-20") {
                return Ok(false);
            }

            // Check operation field
            if obj.get("op").and_then(|v| v.as_str()).is_none() {
                return Ok(false);
            }

            // Check token symbol field
            if obj.get("tick").and_then(|v| v.as_str()).is_none() {
                return Ok(false);
            }
        } else {
            return Ok(false);
        }

        Ok(true)
    }

    /// Parse script content
    pub fn parse_script(script_hex: &str) -> Result<serde_json::Value> {
        let decoded_bytes = hex::decode(script_hex)?;
        let json_str = String::from_utf8(decoded_bytes)?;
        let json_value: serde_json::Value = serde_json::from_str(&json_str)?;
        Ok(json_value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_issue_script() {
        let script = ScriptBuilder::build_issue_script(
            "DRAGON",
            "Dragon Token",
            1000000,
            8,
            "A legendary dragon token",
        )
        .unwrap();

        // Validate script format
        assert!(ScriptBuilder::validate_script(&script).unwrap());

        // Parse script content
        let parsed = ScriptBuilder::parse_script(&script).unwrap();
        assert_eq!(parsed["p"], "KRC-20");
        assert_eq!(parsed["op"], "issue");
        assert_eq!(parsed["tick"], "DRAGON");
        assert_eq!(parsed["name"], "Dragon Token");
        assert_eq!(parsed["max"], "1000000");
        assert_eq!(parsed["dec"], "8");
        assert_eq!(parsed["desc"], "A legendary dragon token");
    }

    #[test]
    fn test_build_send_script() {
        let script = ScriptBuilder::build_send_script(
            "DRAGON",
            "kaspatest:qrf7saw4vlc006mmcpfa29mrgs79ez5glszc4ytx9hq7wdu2a5d4kvl7wz5pz",
            "10000",
        )
        .unwrap();

        assert!(ScriptBuilder::validate_script(&script).unwrap());

        let parsed = ScriptBuilder::parse_script(&script).unwrap();
        assert_eq!(parsed["p"], "KRC-20");
        assert_eq!(parsed["op"], "send");
        assert_eq!(parsed["tick"], "DRAGON");
        assert_eq!(
            parsed["to"],
            "kaspatest:qrf7saw4vlc006mmcpfa29mrgs79ez5glszc4ytx9hq7wdu2a5d4kvl7wz5pz"
        );
        assert_eq!(parsed["amt"], "10000");
    }

    #[test]
    fn test_build_mint_script() {
        let script = ScriptBuilder::build_mint_script(
            "DRAGON",
            "kaspatest:qrf7saw4vlc006mmcpfa29mrgs79ez5glszc4ytx9hq7wdu2a5d4kvl7wz5pz",
            "5000",
        )
        .unwrap();

        assert!(ScriptBuilder::validate_script(&script).unwrap());

        let parsed = ScriptBuilder::parse_script(&script).unwrap();
        assert_eq!(parsed["p"], "KRC-20");
        assert_eq!(parsed["op"], "mint");
        assert_eq!(parsed["tick"], "DRAGON");
        assert_eq!(
            parsed["to"],
            "kaspatest:qrf7saw4vlc006mmcpfa29mrgs79ez5glszc4ytx9hq7wdu2a5d4kvl7wz5pz"
        );
        assert_eq!(parsed["amt"], "5000");
    }

    #[test]
    fn test_script_consistency_with_go() {
        // Test consistency with Go version
        let rust_script = ScriptBuilder::build_issue_script(
            "DRAGON",
            "Dragon Token",
            1000000,
            8,
            "A legendary dragon token",
        )
        .unwrap();

        // Expected Go version script (manually calculated)
        let expected_go_script = "7b2270223a224b52432d3230222c226f70223a226973737565222c227469636b223a22445241474f4e222c226e616d65223a22447261676f6e20546f6b656e222c226d6178223a2231303030303030222c22646563223a2238222c2264657363223a2241206c6567656e6461727920647261676f6e20746f6b656e227d";

        assert_eq!(rust_script, expected_go_script, "Rust script inconsistent with Go script");
    }
}
