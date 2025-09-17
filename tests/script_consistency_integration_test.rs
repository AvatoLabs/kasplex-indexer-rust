use anyhow::Result;
use kaspa_indexer_rust::operations::deploy::DeployOperation;
use kaspa_indexer_rust::operations::issue::IssueOperation;
use kaspa_indexer_rust::operations::mint::MintOperation;
use kaspa_indexer_rust::operations::send::SendOperation;
use kaspa_indexer_rust::utils::script_builder::ScriptBuilder;

/// Script consistency integration test
/// Verify that all operation types generate scripts consistent with Go version
#[cfg(test)]
mod script_consistency_integration_tests {
    use super::*;

    #[test]
    fn test_issue_script_consistency() -> Result<()> {
        println!("🧪 Testing issue script consistency");

        // Use new script builder
        let rust_script = IssueOperation::build_script(
            "DRAGON",
            "Dragon Token",
            1000000,
            8,
            "A legendary dragon token",
        )?;

        // Expected Go version script
        let expected_go_script = "7b2270223a224b52432d3230222c226f70223a226973737565222c227469636b223a22445241474f4e222c226e616d65223a22447261676f6e20546f6b656e222c226d6178223a2231303030303030222c22646563223a2238222c2264657363223a2241206c6567656e6461727920647261676f6e20746f6b656e227d";

        println!("  Rust script: {}", rust_script);
        println!("  Go script:   {}", expected_go_script);

        assert_eq!(rust_script, expected_go_script, "Issue script inconsistency");
        println!("  ✅ Issue script consistency verification passed");

        Ok(())
    }

    #[test]
    fn test_send_script_consistency() -> Result<()> {
        println!("🧪 Testing transfer script consistency");

        let rust_script = SendOperation::build_script(
            "DRAGON",
            "kaspatest:qrf7saw4vlc006mmcpfa29mrgs79ez5glszc4ytx9hq7wdu2a5d4kvl7wz5pz",
            "10000",
        )?;

        // Expected Go version script
        let expected_go_script = "7b2270223a224b52432d3230222c226f70223a2273656e64222c227469636b223a22445241474f4e222c22746f223a226b61737061746573743a7172663773617734766c633030366d6d6370666132396d7267733739657a35676c737a63347974783968713777647532613564346b766c37777a35707a222c22616d74223a223130303030227d";

        println!("  Rust script: {}", rust_script);
        println!("  Go script:   {}", expected_go_script);

        assert_eq!(rust_script, expected_go_script, "Transfer script inconsistency");
        println!("  ✅ Transfer script consistency verification passed");

        Ok(())
    }

    #[test]
    fn test_mint_script_consistency() -> Result<()> {
        println!("🧪 Testing mint script consistency");

        let rust_script = MintOperation::build_script(
            "DRAGON",
            "kaspatest:qrf7saw4vlc006mmcpfa29mrgs79ez5glszc4ytx9hq7wdu2a5d4kvl7wz5pz",
            "5000",
        )?;

        // Expected Go version script
        let expected_go_script = "7b2270223a224b52432d3230222c226f70223a226d696e74222c227469636b223a22445241474f4e222c22746f223a226b61737061746573743a7172663773617734766c633030366d6d6370666132396d7267733739657a35676c737a63347974783968713777647532613564346b766c37777a35707a222c22616d74223a2235303030227d";

        println!("  Rust script: {}", rust_script);
        println!("  Go script:   {}", expected_go_script);

        assert_eq!(rust_script, expected_go_script, "Mint script inconsistency");
        println!("  ✅ Mint script consistency verification passed");

        Ok(())
    }

    #[test]
    fn test_deploy_script_consistency() -> Result<()> {
        println!("🧪 Testing deploy script consistency");

        let rust_script = DeployOperation::build_script(
            "DRAGON",
            "Dragon Token",
            1000000,
            8,
            "A legendary dragon token",
        )?;

        // Expected Go version script
        let expected_go_script = "7b2270223a224b52432d3230222c226f70223a226465706c6f79222c227469636b223a22445241474f4e222c226e616d65223a22447261676f6e20546f6b656e222c226d6178223a2231303030303030222c22646563223a2238222c2264657363223a2241206c6567656e6461727920647261676f6e20746f6b656e227d";

        println!("  Rust script: {}", rust_script);
        println!("  Go script:   {}", expected_go_script);

        assert_eq!(rust_script, expected_go_script, "Deploy script inconsistency");
        println!("  ✅ Deploy script consistency verification passed");

        Ok(())
    }

    #[test]
    fn test_script_builder_consistency() -> Result<()> {
        println!("🧪 Testing script builder consistency");

        // Test all operation types
        let issue_script = ScriptBuilder::build_issue_script(
            "TEST",
            "Test Token",
            1000000,
            8,
            "Test description",
        )?;

        let send_script = ScriptBuilder::build_send_script(
            "TEST",
            "kaspatest:qrf7saw4vlc006mmcpfa29mrgs79ez5glszc4ytx9hq7wdu2a5d4kvl7wz5pz",
            "1000",
        )?;

        let mint_script = ScriptBuilder::build_mint_script(
            "TEST",
            "kaspatest:qrf7saw4vlc006mmcpfa29mrgs79ez5glszc4ytx9hq7wdu2a5d4kvl7wz5pz",
            "500",
        )?;

        let deploy_script = ScriptBuilder::build_deploy_script(
            "TEST",
            "Test Token",
            1000000,
            8,
            "Test description",
        )?;

        // Validate all script formats
        assert!(ScriptBuilder::validate_script(&issue_script)?);
        assert!(ScriptBuilder::validate_script(&send_script)?);
        assert!(ScriptBuilder::validate_script(&mint_script)?);
        assert!(ScriptBuilder::validate_script(&deploy_script)?);

        println!("  ✅ All script format validation passed");

        // Validate script content
        let issue_parsed = ScriptBuilder::parse_script(&issue_script)?;
        assert_eq!(issue_parsed["p"], "KRC-20");
        assert_eq!(issue_parsed["op"], "issue");
        assert_eq!(issue_parsed["tick"], "TEST");

        let send_parsed = ScriptBuilder::parse_script(&send_script)?;
        assert_eq!(send_parsed["p"], "KRC-20");
        assert_eq!(send_parsed["op"], "send");
        assert_eq!(send_parsed["tick"], "TEST");

        println!("  ✅ All script content validation passed");

        Ok(())
    }

    #[test]
    fn test_comprehensive_script_consistency() -> Result<()> {
        println!("🧪 Comprehensive script consistency test");

        // Test multiple different token parameters
        let test_cases = vec![
            ("TEST1", "Test Token 1", 1000000, 6, "Test description 1"),
            ("TEST2", "Test Token 2", 5000000, 18, "Test description 2"),
            ("TEST3", "Test Token 3", 10000000, 8, "Test description 3"),
        ];

        for (tick, name, max_supply, decimals, description) in test_cases {
            println!("  Testing token: {}", tick);

            // Test issue script
            let issue_script =
                IssueOperation::build_script(tick, name, max_supply, decimals, description)?;
            assert!(ScriptBuilder::validate_script(&issue_script)?);

            // Test transfer script
            let send_script = SendOperation::build_script(
                tick,
                "kaspatest:qrf7saw4vlc006mmcpfa29mrgs79ez5glszc4ytx9hq7wdu2a5d4kvl7wz5pz",
                "1000",
            )?;
            assert!(ScriptBuilder::validate_script(&send_script)?);

            // Test mint script
            let mint_script = MintOperation::build_script(
                tick,
                "kaspatest:qrf7saw4vlc006mmcpfa29mrgs79ez5glszc4ytx9hq7wdu2a5d4kvl7wz5pz",
                "500",
            )?;
            assert!(ScriptBuilder::validate_script(&mint_script)?);

            println!("    ✅ {} All operation script validation passed", tick);
        }

        println!("  ✅ Comprehensive script consistency test passed");
        Ok(())
    }

    #[test]
    fn test_script_field_order_consistency() -> Result<()> {
        println!("🧪 Testing script field order consistency");

        // Test issue script field order
        let issue_script = ScriptBuilder::build_issue_script(
            "ORDER",
            "Order Test Token",
            1000000,
            8,
            "Order test description",
        )?;

        let parsed = ScriptBuilder::parse_script(&issue_script)?;

        // Validate field order (according to Go version order)
        if let Some(obj) = parsed.as_object() {
            let keys: Vec<&String> = obj.keys().collect();
            let expected_order = vec!["p", "op", "tick", "name", "max", "dec", "desc"];

            assert_eq!(keys, expected_order, "Field order inconsistent");
            println!("  ✅ Issue script field order correct");
        }

        // Test transfer script field order
        let send_script = ScriptBuilder::build_send_script(
            "ORDER",
            "kaspatest:qrf7saw4vlc006mmcpfa29mrgs79ez5glszc4ytx9hq7wdu2a5d4kvl7wz5pz",
            "1000",
        )?;

        let parsed = ScriptBuilder::parse_script(&send_script)?;

        if let Some(obj) = parsed.as_object() {
            let keys: Vec<&String> = obj.keys().collect();
            let expected_order = vec!["p", "op", "tick", "to", "amt"];

            assert_eq!(keys, expected_order, "Transfer script field order inconsistent");
            println!("  ✅ Transfer script field order correct");
        }

        println!("  ✅ All script field order consistency verification passed");
        Ok(())
    }
}
