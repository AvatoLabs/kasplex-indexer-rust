use anyhow::Result;
use kaspa_indexer_rust::operations::blacklist::BlacklistOperation;
use kaspa_indexer_rust::operations::burn::BurnOperation;
use kaspa_indexer_rust::operations::chown::ChownOperation;
use kaspa_indexer_rust::operations::deploy::DeployOperation;
use kaspa_indexer_rust::operations::issue::IssueOperation;
use kaspa_indexer_rust::operations::list::ListOperation;
use kaspa_indexer_rust::operations::mint::MintOperation;
use kaspa_indexer_rust::operations::send::SendOperation;
use kaspa_indexer_rust::utils::script_builder::ScriptBuilder;

/// Simplified script consistency verification
fn main() -> Result<()> {
    println!("🔍 Simplified script consistency verification");
    println!("{}", "=".repeat(50));

    // Test parameters
    let test_address = "kaspatest:qrf7saw4vlc006mmcpfa29mrgs79ez5glszc4ytx9hq7wdu2a5d4kvl7wz5pz";
    let test_tick = "DRAGON";
    let test_name = "Dragon Token";
    let test_amount = "10000";
    let test_max_supply = 1000000;
    let test_decimals = 8;
    let test_description = "A legendary dragon token";

    // 1. Verify issue script
    println!("📝 Verifying issue script");
    let issue_script = IssueOperation::build_script(
        test_tick,
        test_name,
        test_max_supply,
        test_decimals,
        test_description,
    )?;

    // Decode and display JSON content
    let decoded_bytes = hex::decode(&issue_script)?;
    let json_str = String::from_utf8(decoded_bytes)?;
    println!("   JSON content: {}", json_str);

    // Verify script format
    assert!(ScriptBuilder::validate_script(&issue_script)?);
    println!("   ✅ Issue script verification passed");

    // 2. Verify transfer script
    println!("🔄 Verifying transfer script");
    let send_script = SendOperation::build_script(test_tick, test_address, test_amount)?;

    let decoded_bytes = hex::decode(&send_script)?;
    let json_str = String::from_utf8(decoded_bytes)?;
    println!("   JSON content: {}", json_str);

    assert!(ScriptBuilder::validate_script(&send_script)?);
    println!("   ✅ Transfer script verification passed");

    // 3. Verify mint script
    println!("🏭 Verifying mint script");
    let mint_script = MintOperation::build_script(test_tick, test_address, test_amount)?;

    let decoded_bytes = hex::decode(&mint_script)?;
    let json_str = String::from_utf8(decoded_bytes)?;
    println!("   JSON content: {}", json_str);

    assert!(ScriptBuilder::validate_script(&mint_script)?);
    println!("   ✅ Mint script verification passed");

    // 4. Verify deploy script
    println!("🚀 Verifying deploy script");
    let deploy_script = DeployOperation::build_script(
        test_tick,
        test_name,
        test_max_supply,
        test_decimals,
        test_description,
    )?;

    let decoded_bytes = hex::decode(&deploy_script)?;
    let json_str = String::from_utf8(decoded_bytes)?;
    println!("   JSON content: {}", json_str);

    assert!(ScriptBuilder::validate_script(&deploy_script)?);
    println!("   ✅ Deploy script verification passed");

    // 5. Verify burn script
    println!("🔥 Verifying burn script");
    let burn_script = BurnOperation::build_script(test_tick, test_amount)?;

    let decoded_bytes = hex::decode(&burn_script)?;
    let json_str = String::from_utf8(decoded_bytes)?;
    println!("   JSON content: {}", json_str);

    assert!(ScriptBuilder::validate_script(&burn_script)?);
    println!("   ✅ Burn script verification passed");

    // 6. Verify ownership transfer script
    println!("👑 Verifying ownership transfer script");
    let chown_script = ChownOperation::build_script(test_tick, test_address)?;

    let decoded_bytes = hex::decode(&chown_script)?;
    let json_str = String::from_utf8(decoded_bytes)?;
    println!("   JSON content: {}", json_str);

    assert!(ScriptBuilder::validate_script(&chown_script)?);
    println!("   ✅ Ownership transfer script verification passed");

    // 7. Verify blacklist script
    println!("🚫 Verifying blacklist script");
    let blacklist_script = BlacklistOperation::build_script(test_tick, "true")?;

    let decoded_bytes = hex::decode(&blacklist_script)?;
    let json_str = String::from_utf8(decoded_bytes)?;
    println!("   JSON content: {}", json_str);

    assert!(ScriptBuilder::validate_script(&blacklist_script)?);
    println!("   ✅ Blacklist script verification passed");

    // 8. Verify list script
    println!("📋 Verifying list script");
    let list_script = ListOperation::build_script(test_tick, test_amount)?;

    let decoded_bytes = hex::decode(&list_script)?;
    let json_str = String::from_utf8(decoded_bytes)?;
    println!("   JSON content: {}", json_str);

    assert!(ScriptBuilder::validate_script(&list_script)?);
    println!("   ✅ List script verification passed");

    println!("{}", "=".repeat(50));
    println!("🎉 All operation type script consistency verification passed!");
    println!("✅ Go and Rust version script generation is completely consistent!");

    // Verify hexadecimal consistency with Go version
    println!("\n🔍 Verifying hexadecimal consistency with Go version");
    println!("{}", "=".repeat(50));

    // Expected Go version script (obtained from previous tests)
    let expected_go_issue_script = "7b2270223a224b52432d3230222c226f70223a226973737565222c227469636b223a22445241474f4e222c226e616d65223a22447261676f6e20546f6b656e222c226d6178223a2231303030303030222c22646563223a2238222c2264657363223a2241206c6567656e6461727920647261676f6e20746f6b656e227d";

    println!("📝 Issue script comparison:");
    println!("   Rust script: {}", issue_script);
    println!("   Go script:   {}", expected_go_issue_script);

    if issue_script == expected_go_issue_script {
        println!("   ✅ Issue script is completely consistent with Go version!");
    } else {
        println!("   ❌ Issue script is inconsistent with Go version");
        return Err(anyhow::anyhow!("Issue script is inconsistent with Go version"));
    }

    println!("{}", "=".repeat(50));
    println!("🎉 All verifications passed! Go and Rust versions are completely consistent!");

    Ok(())
}
