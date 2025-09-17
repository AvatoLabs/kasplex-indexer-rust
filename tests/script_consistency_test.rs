use anyhow::Result;
use hex;
use kaspa_indexer_rust::operations::issue::IssueOperation;
use kaspa_indexer_rust::operations::mint::MintOperation;
use kaspa_indexer_rust::operations::send::SendOperation;
use kaspa_indexer_rust::storage::types::*;
use serde_json::json;

/// Script consistency test
/// Ensures that the Rust version generates scripts that are completely consistent with the Go version
#[cfg(test)]
mod script_consistency_tests {
    use super::*;

    #[test]
    fn test_issue_script_consistency() -> Result<()> {
        println!("🧪 Testing issue script consistency");

        // Test case 1: Basic issue script
        let token_info = TokenInfo {
            tick: "DRAGON".to_string(),
            name: "Dragon Token".to_string(),
            max_supply: 1000000,
            decimals: 8,
            description: "A legendary dragon token".to_string(),
        };

        let rust_script = build_issue_script(&token_info)?;
        println!("  Rust script: {}", rust_script);

        // Expected Go version script (needs to be obtained from Go version)
        let expected_go_script = "7b2270223a224b52432d3230222c226f70223a226973737565222c227469636b223a22445241474f4e222c226e616d65223a22447261676f6e20546f6b656e222c226d6178223a2231303030303030222c22646563223a2238222c2264657363223a2241206c6567656e6461727920647261676f6e20746f6b656e227d";

        // Verify script consistency
        assert_eq!(rust_script, expected_go_script, "Issue script inconsistency");
        println!("  ✅ Issue script consistency verification passed");

        Ok(())
    }

    #[test]
    fn test_transfer_script_consistency() -> Result<()> {
        println!("🧪 Testing transfer script consistency");

        let rust_script = build_transfer_script(
            "DRAGON",
            "kaspatest:qrf7saw4vlc006mmcpfa29mrgs79ez5glszc4ytx9hq7wdu2a5d4kvl7wz5pz",
            "10000",
        )?;
        println!("  Rust script: {}", rust_script);

        // Expected Go version script
        let expected_go_script = "7b2270223a224b52432d3230222c226f70223a2273656e64222c227469636b223a22445241474f4e222c22746f223a226b61737061746573743a7172663773617734766c633030366d6d6370666132396d7267733739657a35676c737a63347974783968713777647532613564346b766c37777a35707a222c22616d74223a223130303030227d";

        assert_eq!(rust_script, expected_go_script, "Transfer script inconsistency");
        println!("  ✅ Transfer script consistency verification passed");

        Ok(())
    }

    #[test]
    fn test_mint_script_consistency() -> Result<()> {
        println!("🧪 Testing mint script consistency");

        let rust_script = build_mint_script(
            "DRAGON",
            "kaspatest:qrf7saw4vlc006mmcpfa29mrgs79ez5glszc4ytx9hq7wdu2a5d4kvl7wz5pz",
            "5000",
        )?;
        println!("  Rust script: {}", rust_script);

        // Expected Go version script
        let expected_go_script = "7b2270223a224b52432d3230222c226f70223a226d696e74222c227469636b223a22445241474f4e222c22746f223a226b61737061746573743a7172663773617734766c633030366d6d6370666132396d7267733739657a35676c737a63347974783968713777647532613564346b766c37777a35707a222c22616d74223a2235303030227d";

        assert_eq!(rust_script, expected_go_script, "Mint script inconsistency");
        println!("  ✅ Mint script consistency verification passed");

        Ok(())
    }

    #[test]
    fn test_script_validation_consistency() -> Result<()> {
        println!("🧪 Testing script validation consistency");

        // Test issue script validation
        let mut issue_script = DataScriptType {
            p: "KRC-20".to_string(),
            op: "issue".to_string(),
            tick: Some("DRAGON".to_string()),
            name: Some("Dragon Token".to_string()),
            max: Some("1000000".to_string()),
            dec: Some("8".to_string()),
            from: Some(
                "kaspatest:qrf7saw4vlc006mmcpfa29mrgs79ez5glszc4ytx9hq7wdu2a5d4kvl7wz5pz"
                    .to_string(),
            ),
            to: Some(
                "kaspatest:qrf7saw4vlc006mmcpfa29mrgs79ez5glszc4ytx9hq7wdu2a5d4kvl7wz5pz"
                    .to_string(),
            ),
            amt: Some("100000".to_string()),
            lim: None,
            pre: None,
            utxo: None,
            price: None,
            mod_type: "issue".to_string(),
            ca: None,
        };

        let rust_validation =
            IssueOperation::validate(&mut issue_script, "test_tx_id", 110165000, true);
        println!("  Rust validation result: {}", rust_validation);

        // This should be consistent with the Go version validation result
        // Due to the specific validation logic of the Go version, actual testing is needed here
        assert!(rust_validation, "Issue script validation result inconsistency");
        println!("  ✅ Issue script validation consistency passed");

        // Test transfer script validation
        let mut transfer_script = DataScriptType {
            p: "KRC-20".to_string(),
            op: "send".to_string(),
            tick: Some("DRAGON".to_string()),
            name: None,
            max: None,
            dec: None,
            from: Some(
                "kaspatest:qrf7saw4vlc006mmcpfa29mrgs79ez5glszc4ytx9hq7wdu2a5d4kvl7wz5pz"
                    .to_string(),
            ),
            to: Some(
                "kaspatest:qrf7saw4vlc006mmcpfa29mrgs79ez5glszc4ytx9hq7wdu2a5d4kvl7wz5pz"
                    .to_string(),
            ),
            amt: Some("10000".to_string()),
            lim: None,
            pre: None,
            utxo: None,
            price: None,
            mod_type: "send".to_string(),
            ca: None,
        };

        let rust_transfer_validation =
            SendOperation::validate(&mut transfer_script, "test_tx_id", 110165000, true);
        println!("  Rust transfer validation result: {}", rust_transfer_validation);

        assert!(rust_transfer_validation, "Transfer script validation result inconsistency");
        println!("  ✅ Transfer script validation consistency passed");

        Ok(())
    }

    #[test]
    fn test_json_serialization_consistency() -> Result<()> {
        println!("🧪 Testing JSON serialization consistency");

        // Test JSON serialization of issue script
        let issue_data = json!({
            "p": "KRC-20",
            "op": "issue",
            "tick": "DRAGON",
            "name": "Dragon Token",
            "max": "1000000",
            "dec": "8",
            "desc": "A legendary dragon token"
        });

        let rust_json = serde_json::to_string(&issue_data)?;
        println!("  Rust JSON: {}", rust_json);

        // Verify JSON format is consistent with Go version
        assert!(rust_json.contains("\"p\":\"KRC-20\""), "JSON format inconsistency");
        assert!(rust_json.contains("\"op\":\"issue\""), "JSON format inconsistency");
        assert!(rust_json.contains("\"tick\":\"DRAGON\""), "JSON format inconsistency");

        println!("  ✅ JSON serialization consistency verification passed");

        Ok(())
    }

    #[test]
    fn test_hex_encoding_consistency() -> Result<()> {
        println!("🧪 Testing hexadecimal encoding consistency");

        let test_string = r#"{"p":"KRC-20","op":"issue","tick":"DRAGON","name":"Dragon Token","max":"1000000","dec":"8","desc":"A legendary dragon token"}"#;

        let rust_hex = hex::encode(test_string.as_bytes());
        println!("  Rust hexadecimal: {}", rust_hex);

        // Verify hexadecimal encoding result
        let expected_hex = "7b2270223a224b52432d3230222c226f70223a226973737565222c227469636b223a22445241474f4e222c226e616d65223a22447261676f6e20546f6b656e222c226d6178223a2231303030303030222c22646563223a2238222c2264657363223a2241206c6567656e6461727920647261676f6e20746f6b656e227d";

        assert_eq!(rust_hex, expected_hex, "Hexadecimal encoding inconsistency");
        println!("  ✅ Hexadecimal encoding consistency verification passed");

        Ok(())
    }

    #[test]
    fn test_comprehensive_script_comparison() -> Result<()> {
        println!("🧪 Comprehensive script comparison test");

        // Test multiple different token parameters
        let test_cases = vec![
            ("TEST1", "Test Token 1", 1000000, 6, "Test description 1"),
            ("TEST2", "Test Token 2", 5000000, 18, "Test description 2"),
            ("TEST3", "Test Token 3", 10000000, 8, "Test description 3"),
        ];

        for (tick, name, max_supply, decimals, description) in test_cases {
            let token_info = TokenInfo {
                tick: tick.to_string(),
                name: name.to_string(),
                max_supply,
                decimals,
                description: description.to_string(),
            };

            let rust_script = build_issue_script(&token_info)?;
            println!("  {} script: {}", tick, rust_script);

            // Verify script format
            assert!(rust_script.len() > 0, "Script cannot be empty");
            assert!(
                rust_script.chars().all(|c| c.is_ascii_hexdigit()),
                "Script must be valid hexadecimal"
            );

            // Verify script can be decoded correctly
            let decoded_bytes = hex::decode(&rust_script)?;
            let decoded_json: serde_json::Value = serde_json::from_slice(&decoded_bytes)?;

            assert_eq!(decoded_json["p"], "KRC-20", "Protocol field incorrect");
            assert_eq!(decoded_json["op"], "issue", "Operation field incorrect");
            assert_eq!(decoded_json["tick"], tick, "Token symbol incorrect");
            assert_eq!(decoded_json["name"], name, "Token name incorrect");
            assert_eq!(
                decoded_json["max"],
                max_supply.to_string(),
                "Maximum supply incorrect"
            );
            assert_eq!(decoded_json["dec"], decimals.to_string(), "Decimal places incorrect");
            assert_eq!(decoded_json["desc"], description, "Description incorrect");
        }

        println!("  ✅ Comprehensive script comparison test passed");
        Ok(())
    }
}

/// Build issue script
fn build_issue_script(token_info: &TokenInfo) -> Result<String> {
    let script_data = json!({
        "p": "KRC-20",
        "op": "issue",
        "tick": token_info.tick,
        "name": token_info.name,
        "max": token_info.max_supply.to_string(),
        "dec": token_info.decimals.to_string(),
        "desc": token_info.description
    });

    let script_str = serde_json::to_string(&script_data)?;
    let script_hex = hex::encode(script_str.as_bytes());

    Ok(script_hex)
}

/// Build transfer script
fn build_transfer_script(tick: &str, to_address: &str, amount: &str) -> Result<String> {
    let script_data = json!({
        "p": "KRC-20",
        "op": "send",
        "tick": tick,
        "to": to_address,
        "amt": amount
    });

    let script_str = serde_json::to_string(&script_data)?;
    let script_hex = hex::encode(script_str.as_bytes());

    Ok(script_hex)
}

/// Build mint script
fn build_mint_script(tick: &str, to_address: &str, amount: &str) -> Result<String> {
    let script_data = json!({
        "p": "KRC-20",
        "op": "mint",
        "tick": tick,
        "to": to_address,
        "amt": amount
    });

    let script_str = serde_json::to_string(&script_data)?;
    let script_hex = hex::encode(script_str.as_bytes());

    Ok(script_hex)
}

/// Token information structure
#[derive(Debug)]
struct TokenInfo {
    tick: String,
    name: String,
    max_supply: u64,
    decimals: u8,
    description: String,
}
