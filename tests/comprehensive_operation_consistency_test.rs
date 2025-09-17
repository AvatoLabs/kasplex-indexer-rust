use anyhow::Result;
use kaspa_indexer_rust::operations::blacklist::BlacklistOperation;
use kaspa_indexer_rust::operations::burn::BurnOperation;
use kaspa_indexer_rust::operations::chown::ChownOperation;
use kaspa_indexer_rust::operations::deploy::DeployOperation;
use kaspa_indexer_rust::operations::issue::IssueOperation;
use kaspa_indexer_rust::operations::list::ListOperation;
use kaspa_indexer_rust::operations::mint::MintOperation;
use kaspa_indexer_rust::operations::send::SendOperation;
use kaspa_indexer_rust::storage::types::*;
use kaspa_indexer_rust::utils::script_builder::ScriptBuilder;
use kaspa_indexer_rust::utils::script_parser::ScriptParser;

/// Comprehensive operation consistency test
/// Cover all operation types, verify script generation and parsing logic consistency
#[cfg(test)]
mod comprehensive_operation_consistency_tests {
    use super::*;

    /// Test script generation consistency for all operation types
    #[test]
    fn test_all_operation_script_consistency() -> Result<()> {
        println!("🧪 Testing script consistency for all operation types");

        // Test issue operation
        test_issue_operation_consistency()?;

        // Test transfer operation
        test_send_operation_consistency()?;

        // Test mint operation
        test_mint_operation_consistency()?;

        // Test deploy operation
        test_deploy_operation_consistency()?;

        // Test burn operation
        test_burn_operation_consistency()?;

        // Test ownership transfer operation
        test_chown_operation_consistency()?;

        // Test blacklist operation
        test_blacklist_operation_consistency()?;

        // Test list operation
        test_list_operation_consistency()?;

        println!("  ✅ All operation type script consistency verification passed");
        Ok(())
    }

    /// Test issue operation consistency
    fn test_issue_operation_consistency() -> Result<()> {
        println!("  📝 Testing issue operation");

        let rust_script = IssueOperation::build_script(
            "DRAGON",
            "Dragon Token",
            1000000,
            8,
            "A legendary dragon token",
        )?;

        // Expected Go version script
        let expected_go_script = "7b2270223a224b52432d3230222c226f70223a226973737565222c227469636b223a22445241474f4e222c226e616d65223a22447261676f6e20546f6b656e222c226d6178223a2231303030303030222c22646563223a2238222c2264657363223a2241206c6567656e6461727920647261676f6e20746f6b656e227d";

        assert_eq!(rust_script, expected_go_script, "Issue script inconsistency");

        // Verify script parsing
        let parsed = ScriptBuilder::parse_script(&rust_script)?;
        assert_eq!(parsed["p"], "KRC-20");
        assert_eq!(parsed["op"], "issue");
        assert_eq!(parsed["tick"], "DRAGON");
        assert_eq!(parsed["name"], "Dragon Token");
        assert_eq!(parsed["max"], "1000000");
        assert_eq!(parsed["dec"], "8");
        assert_eq!(parsed["desc"], "A legendary dragon token");

        println!("    ✅ Issue operation consistency verification passed");
        Ok(())
    }

    /// Test transfer operation consistency
    fn test_send_operation_consistency() -> Result<()> {
        println!("  🔄 Testing transfer operation");

        let rust_script = SendOperation::build_script(
            "DRAGON",
            "kaspatest:qrf7saw4vlc006mmcpfa29mrgs79ez5glszc4ytx9hq7wdu2a5d4kvl7wz5pz",
            "10000",
        )?;

        // Expected Go version script
        let expected_go_script = "7b2270223a224b52432d3230222c226f70223a2273656e64222c227469636b223a22445241474f4e222c22746f223a226b61737061746573743a7172663773617734766c633030366d6d6370666132396d7267733739657a35676c737a63347974783968713777647532613564346b766c37777a35707a222c22616d74223a223130303030227d";

        assert_eq!(rust_script, expected_go_script, "Transfer script inconsistency");

        // Verify script parsing
        let parsed = ScriptBuilder::parse_script(&rust_script)?;
        assert_eq!(parsed["p"], "KRC-20");
        assert_eq!(parsed["op"], "send");
        assert_eq!(parsed["tick"], "DRAGON");
        assert_eq!(
            parsed["to"],
            "kaspatest:qrf7saw4vlc006mmcpfa29mrgs79ez5glszc4ytx9hq7wdu2a5d4kvl7wz5pz"
        );
        assert_eq!(parsed["amt"], "10000");

        println!("    ✅ Transfer operation consistency verification passed");
        Ok(())
    }

    /// Test mint operation consistency
    fn test_mint_operation_consistency() -> Result<()> {
        println!("  🏭 Testing mint operation");

        let rust_script = MintOperation::build_script(
            "DRAGON",
            "kaspatest:qrf7saw4vlc006mmcpfa29mrgs79ez5glszc4ytx9hq7wdu2a5d4kvl7wz5pz",
            "5000",
        )?;

        // Expected Go version script
        let expected_go_script = "7b2270223a224b52432d3230222c226f70223a226d696e74222c227469636b223a22445241474f4e222c22746f223a226b61737061746573743a7172663773617734766c633030366d6d6370666132396d7267733739657a35676c737a63347974783968713777647532613564346b766c37777a35707a222c22616d74223a2235303030227d";

        assert_eq!(rust_script, expected_go_script, "Mint script inconsistency");

        // Verify script parsing
        let parsed = ScriptBuilder::parse_script(&rust_script)?;
        assert_eq!(parsed["p"], "KRC-20");
        assert_eq!(parsed["op"], "mint");
        assert_eq!(parsed["tick"], "DRAGON");
        assert_eq!(
            parsed["to"],
            "kaspatest:qrf7saw4vlc006mmcpfa29mrgs79ez5glszc4ytx9hq7wdu2a5d4kvl7wz5pz"
        );
        assert_eq!(parsed["amt"], "5000");

        println!("    ✅ Mint operation consistency verification passed");
        Ok(())
    }

    /// Test deploy operation consistency
    fn test_deploy_operation_consistency() -> Result<()> {
        println!("  🚀 Testing deploy operation");

        let rust_script = DeployOperation::build_script(
            "DRAGON",
            "Dragon Token",
            1000000,
            8,
            "A legendary dragon token",
        )?;

        // Expected Go version script
        let expected_go_script = "7b2270223a224b52432d3230222c226f70223a226465706c6f79222c227469636b223a22445241474f4e222c226e616d65223a22447261676f6e20546f6b656e222c226d6178223a2231303030303030222c22646563223a2238222c2264657363223a2241206c6567656e6461727920647261676f6e20746f6b656e227d";

        assert_eq!(rust_script, expected_go_script, "Deploy script inconsistency");

        // Verify script parsing
        let parsed = ScriptBuilder::parse_script(&rust_script)?;
        assert_eq!(parsed["p"], "KRC-20");
        assert_eq!(parsed["op"], "deploy");
        assert_eq!(parsed["tick"], "DRAGON");
        assert_eq!(parsed["name"], "Dragon Token");
        assert_eq!(parsed["max"], "1000000");
        assert_eq!(parsed["dec"], "8");
        assert_eq!(parsed["desc"], "A legendary dragon token");

        println!("    ✅ Deploy operation consistency verification passed");
        Ok(())
    }

    /// Test burn operation consistency
    fn test_burn_operation_consistency() -> Result<()> {
        println!("  🔥 Testing burn operation");

        let rust_script = BurnOperation::build_script("DRAGON", "1000")?;

        // Expected Go version script
        let expected_go_script = "7b2270223a224b52432d3230222c226f70223a226275726e222c227469636b223a22445241474f4e222c22616d74223a2231303030227d";

        assert_eq!(rust_script, expected_go_script, "Burn script inconsistency");

        // Verify script parsing
        let parsed = ScriptBuilder::parse_script(&rust_script)?;
        assert_eq!(parsed["p"], "KRC-20");
        assert_eq!(parsed["op"], "burn");
        assert_eq!(parsed["tick"], "DRAGON");
        assert_eq!(parsed["amt"], "1000");

        println!("    ✅ Burn operation consistency verification passed");
        Ok(())
    }

    /// Test ownership transfer operation consistency
    fn test_chown_operation_consistency() -> Result<()> {
        println!("  👑 Testing ownership transfer operation");

        let rust_script = ChownOperation::build_script(
            "DRAGON",
            "kaspatest:qrf7saw4vlc006mmcpfa29mrgs79ez5glszc4ytx9hq7wdu2a5d4kvl7wz5pz",
        )?;

        // Expected Go version script
        let expected_go_script = "7b2270223a224b52432d3230222c226f70223a2263686f776e222c227469636b223a22445241474f4e222c22746f223a226b61737061746573743a7172663773617734766c633030366d6d6370666132396d7267733739657a35676c737a63347974783968713777647532613564346b766c37777a35707a227d";

        assert_eq!(rust_script, expected_go_script, "Ownership transfer script inconsistency");

        // Verify script parsing
        let parsed = ScriptBuilder::parse_script(&rust_script)?;
        assert_eq!(parsed["p"], "KRC-20");
        assert_eq!(parsed["op"], "chown");
        assert_eq!(parsed["tick"], "DRAGON");
        assert_eq!(
            parsed["to"],
            "kaspatest:qrf7saw4vlc006mmcpfa29mrgs79ez5glszc4ytx9hq7wdu2a5d4kvl7wz5pz"
        );

        println!("    ✅ Ownership transfer operation consistency verification passed");
        Ok(())
    }

    /// Test blacklist operation consistency
    fn test_blacklist_operation_consistency() -> Result<()> {
        println!("  🚫 Testing blacklist operation");

        let rust_script = BlacklistOperation::build_script("DRAGON", "true")?;

        // Expected Go version script
        let expected_go_script = "7b2270223a224b52432d3230222c226f70223a22626c61636b6c697374222c227469636b223a22445241474f4e222c22626c61636b6c697374223a2274727565227d";

        assert_eq!(rust_script, expected_go_script, "Blacklist script inconsistency");

        // Verify script parsing
        let parsed = ScriptBuilder::parse_script(&rust_script)?;
        assert_eq!(parsed["p"], "KRC-20");
        assert_eq!(parsed["op"], "blacklist");
        assert_eq!(parsed["tick"], "DRAGON");
        assert_eq!(parsed["blacklist"], "true");

        println!("    ✅ Blacklist operation consistency verification passed");
        Ok(())
    }

    /// Test list operation consistency
    fn test_list_operation_consistency() -> Result<()> {
        println!("  📋 Testing list operation");

        let rust_script = ListOperation::build_script("DRAGON", "1000")?;

        // Expected Go version script
        let expected_go_script = "7b2270223a224b52432d3230222c226f70223a226c697374222c227469636b223a22445241474f4e222c226c697374223a2231303030227d";

        assert_eq!(rust_script, expected_go_script, "List script inconsistency");

        // Verify script parsing
        let parsed = ScriptBuilder::parse_script(&rust_script)?;
        assert_eq!(parsed["p"], "KRC-20");
        assert_eq!(parsed["op"], "list");
        assert_eq!(parsed["tick"], "DRAGON");
        assert_eq!(parsed["list"], "1000");

        println!("    ✅ List operation consistency verification passed");
        Ok(())
    }

    /// Test script parsing logic consistency
    #[test]
    fn test_script_parsing_consistency() -> Result<()> {
        println!("🧪 Testing script parsing logic consistency");

        // Test script parsing for all operation types
        let test_cases = vec![
            (
                "issue",
                IssueOperation::build_script("TEST", "Test Token", 1000000, 8, "Test description")?,
            ),
            (
                "send",
                SendOperation::build_script(
                    "TEST",
                    "kaspatest:qrf7saw4vlc006mmcpfa29mrgs79ez5glszc4ytx9hq7wdu2a5d4kvl7wz5pz",
                    "1000",
                )?,
            ),
            (
                "mint",
                MintOperation::build_script(
                    "TEST",
                    "kaspatest:qrf7saw4vlc006mmcpfa29mrgs79ez5glszc4ytx9hq7wdu2a5d4kvl7wz5pz",
                    "500",
                )?,
            ),
            (
                "deploy",
                DeployOperation::build_script(
                    "TEST",
                    "Test Token",
                    1000000,
                    8,
                    "Test description",
                )?,
            ),
            ("burn", BurnOperation::build_script("TEST", "1000")?),
            (
                "chown",
                ChownOperation::build_script(
                    "TEST",
                    "kaspatest:qrf7saw4vlc006mmcpfa29mrgs79ez5glszc4ytx9hq7wdu2a5d4kvl7wz5pz",
                )?,
            ),
            (
                "blacklist",
                BlacklistOperation::build_script("TEST", "true")?,
            ),
            ("list", ListOperation::build_script("TEST", "1000")?),
        ];

        for (op_type, script_hex) in test_cases {
            println!("  Testing {} operation parsing", op_type);

            // Verify script format
            assert!(
                ScriptBuilder::validate_script(&script_hex)?,
                "{} script format verification failed",
                op_type
            );

            // Parse script content
            let parsed = ScriptBuilder::parse_script(&script_hex)?;

            // Verify basic fields
            assert_eq!(parsed["p"], "KRC-20", "{} protocol field incorrect", op_type);
            assert_eq!(parsed["op"], op_type, "{} operation field incorrect", op_type);
            assert_eq!(parsed["tick"], "TEST", "{} token symbol incorrect", op_type);

            // Verify operation-specific fields
            match op_type {
                "issue" | "deploy" => {
                    assert_eq!(parsed["name"], "Test Token");
                    assert_eq!(parsed["max"], "1000000");
                    assert_eq!(parsed["dec"], "8");
                    assert_eq!(parsed["desc"], "Test description");
                }
                "send" | "mint" => {
                    assert_eq!(
                        parsed["to"],
                        "kaspatest:qrf7saw4vlc006mmcpfa29mrgs79ez5glszc4ytx9hq7wdu2a5d4kvl7wz5pz"
                    );
                    assert_eq!(
                        parsed["amt"],
                        if op_type == "send" { "1000" } else { "500" }
                    );
                }
                "burn" => {
                    assert_eq!(parsed["amt"], "1000");
                }
                "chown" => {
                    assert_eq!(
                        parsed["to"],
                        "kaspatest:qrf7saw4vlc006mmcpfa29mrgs79ez5glszc4ytx9hq7wdu2a5d4kvl7wz5pz"
                    );
                }
                "blacklist" => {
                    assert_eq!(parsed["blacklist"], "true");
                }
                "list" => {
                    assert_eq!(parsed["list"], "1000");
                }
                _ => {}
            }

            println!("    ✅ {} operation parsing verification passed", op_type);
        }

        println!("  ✅ All operation type parsing logic consistency verification passed");
        Ok(())
    }

    /// Test script validation logic consistency
    #[test]
    fn test_script_validation_consistency() -> Result<()> {
        println!("🧪 Testing script validation logic consistency");

        // Test valid scripts
        let valid_scripts = vec![
            IssueOperation::build_script("VALID", "Valid Token", 1000000, 8, "Valid description")?,
            SendOperation::build_script(
                "VALID",
                "kaspatest:qrf7saw4vlc006mmcpfa29mrgs79ez5glszc4ytx9hq7wdu2a5d4kvl7wz5pz",
                "1000",
            )?,
            MintOperation::build_script(
                "VALID",
                "kaspatest:qrf7saw4vlc006mmcpfa29mrgs79ez5glszc4ytx9hq7wdu2a5d4kvl7wz5pz",
                "500",
            )?,
        ];

        for script in valid_scripts {
            assert!(ScriptBuilder::validate_script(&script)?, "Valid script validation failed");
        }

        // Test invalid scripts
        let invalid_scripts = vec![
            "invalid_hex",                                                // Invalid hexadecimal
            "7b2270223a224b52432d3230227d",                               // Missing required fields
            "7b2270223a224b52432d3230222c226f70223a22696e76616c6964227d", // Invalid operation type
        ];

        for script in invalid_scripts {
            assert!(
                !ScriptBuilder::validate_script(script).unwrap_or(false),
                "Invalid script validation should fail"
            );
        }

        println!("  ✅ Script validation logic consistency verification passed");
        Ok(())
    }

    /// Test field order consistency
    #[test]
    fn test_field_order_consistency() -> Result<()> {
        println!("🧪 Testing field order consistency");

        // Test field order for all operation types
        let test_cases = vec![
            (
                "issue",
                vec!["p", "op", "tick", "name", "max", "dec", "desc"],
            ),
            ("send", vec!["p", "op", "tick", "to", "amt"]),
            ("mint", vec!["p", "op", "tick", "to", "amt"]),
            (
                "deploy",
                vec!["p", "op", "tick", "name", "max", "dec", "desc"],
            ),
            ("burn", vec!["p", "op", "tick", "amt"]),
            ("chown", vec!["p", "op", "tick", "to"]),
            ("blacklist", vec!["p", "op", "tick", "blacklist"]),
            ("list", vec!["p", "op", "tick", "list"]),
        ];

        for (op_type, expected_fields) in test_cases {
            println!("  Testing {} operation field order", op_type);

            let script = match op_type {
                "issue" => IssueOperation::build_script(
                    "ORDER",
                    "Order Token",
                    1000000,
                    8,
                    "Order description",
                )?,
                "send" => SendOperation::build_script(
                    "ORDER",
                    "kaspatest:qrf7saw4vlc006mmcpfa29mrgs79ez5glszc4ytx9hq7wdu2a5d4kvl7wz5pz",
                    "1000",
                )?,
                "mint" => MintOperation::build_script(
                    "ORDER",
                    "kaspatest:qrf7saw4vlc006mmcpfa29mrgs79ez5glszc4ytx9hq7wdu2a5d4kvl7wz5pz",
                    "500",
                )?,
                "deploy" => DeployOperation::build_script(
                    "ORDER",
                    "Order Token",
                    1000000,
                    8,
                    "Order description",
                )?,
                "burn" => BurnOperation::build_script("ORDER", "1000")?,
                "chown" => ChownOperation::build_script(
                    "ORDER",
                    "kaspatest:qrf7saw4vlc006mmcpfa29mrgs79ez5glszc4ytx9hq7wdu2a5d4kvl7wz5pz",
                )?,
                "blacklist" => BlacklistOperation::build_script("ORDER", "true")?,
                "list" => ListOperation::build_script("ORDER", "1000")?,
                _ => continue,
            };

            let parsed = ScriptBuilder::parse_script(&script)?;

            if let Some(obj) = parsed.as_object() {
                let actual_fields: Vec<&String> = obj.keys().collect();
                assert_eq!(
                    actual_fields, expected_fields,
                    "{} operation field order inconsistent",
                    op_type
                );
            }

            println!("    ✅ {} operation field order correct", op_type);
        }

        println!("  ✅ All operation type field order consistency verification passed");
        Ok(())
    }

    /// Test edge cases
    #[test]
    fn test_edge_cases() -> Result<()> {
        println!("🧪 Testing edge cases");

        // Test empty string
        let empty_script = ScriptBuilder::build_issue_script("", "", 0, 0, "")?;
        assert!(ScriptBuilder::validate_script(&empty_script)?);

        // Test maximum value
        let max_script = ScriptBuilder::build_issue_script(
            "MAX",
            "Maximum Token",
            u64::MAX,
            255,
            "Maximum description with very long text that might cause issues",
        )?;
        assert!(ScriptBuilder::validate_script(&max_script)?);

        // Test special characters
        let special_script = ScriptBuilder::build_issue_script(
            "SPECIAL",
            "Token with \"quotes\" and \\backslashes\\",
            1000000,
            8,
            "Description with special chars: !@#$%^&*()",
        )?;
        assert!(ScriptBuilder::validate_script(&special_script)?);

        println!("  ✅ Edge case test passed");
        Ok(())
    }
}
