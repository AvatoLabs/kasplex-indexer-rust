use kaspa_indexer_rust::operations::*;
use kaspa_indexer_rust::storage::types::*;

#[test]
fn test_deploy_operation_validation() {
    let mut script = DataScriptType {
        p: "KRC-20".to_string(),
        op: "deploy".to_string(),
        from: Some("kaspa:test_address".to_string()),
        to: Some("kaspa:test_address".to_string()),
        tick: Some("TEST".to_string()),
        max: Some("1000000".to_string()),
        lim: Some("1000".to_string()),
        pre: Some("0".to_string()),
        dec: Some("8".to_string()),
        amt: None,
        utxo: None,
        price: None,
        mod_type: "".to_string(),
        name: None,
        ca: None,
    };

    let result = DeployOperation::validate(&mut script, "test_tx_id", 110165001, false);
    assert!(result, "Deploy operation validation should pass");
    assert_eq!(script.p, "KRC-20");
    assert_eq!(script.op, "deploy");
}

#[test]
fn test_mint_operation_validation() {
    let mut script = DataScriptType {
        p: "KRC-20".to_string(),
        op: "mint".to_string(),
        from: Some("kaspa:test_address".to_string()),
        to: Some("kaspa:test_address".to_string()),
        tick: Some("TEST".to_string()),
        max: None,
        lim: None,
        pre: None,
        dec: None,
        amt: None,
        utxo: None,
        price: None,
        mod_type: "".to_string(),
        name: None,
        ca: None,
    };

    let result = MintOperation::validate(&mut script, "test_tx_id", 110165001, false);
    assert!(result, "Mint operation validation should pass");
    assert_eq!(script.p, "KRC-20");
    assert_eq!(script.op, "mint");
}

#[test]
fn test_transfer_operation_validation() {
    let mut script = DataScriptType {
        p: "KRC-20".to_string(),
        op: "transfer".to_string(),
        from: Some("kaspa:from_address".to_string()),
        to: Some("kaspa:to_address".to_string()),
        tick: Some("TEST".to_string()),
        max: None,
        lim: None,
        pre: None,
        dec: None,
        amt: Some("100".to_string()),
        utxo: None,
        price: None,
        mod_type: "".to_string(),
        name: None,
        ca: None,
    };

    let result = TransferOperation::validate(&mut script, "test_tx_id", 110165001, false);
    assert!(result, "Transfer operation validation should pass");
    assert_eq!(script.p, "KRC-20");
    assert_eq!(script.op, "transfer");
}

#[test]
fn test_operation_fees() {
    // Test fees for various operations
    assert_eq!(DeployOperation::fee_least(110165001), 100000000000);
    assert_eq!(MintOperation::fee_least(110165001), 100000000);
    assert_eq!(TransferOperation::fee_least(110165001), 0);
}

#[test]
fn test_validation_functions() {
    // Test tick validation
    let mut valid_tick = "TEST".to_string();
    assert!(validate_tick(&mut valid_tick), "Valid tick should pass");

    let mut empty_tick = "".to_string();
    assert!(!validate_tick(&mut empty_tick), "Empty tick should fail");

    let mut long_tick = "TOOLONGTICK".to_string();
    assert!(!validate_tick(&mut long_tick), "Too long tick should fail");

    // Test tx_id validation
    let mut valid_tx_id =
        "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string();
    assert!(validate_tx_id(&mut valid_tx_id), "Valid tx_id should pass");

    let mut invalid_tx_id = "invalid".to_string();
    assert!(
        !validate_tx_id(&mut invalid_tx_id),
        "Invalid tx_id should fail"
    );

    // Test amount validation
    let mut valid_amount = "1000".to_string();
    assert!(
        validate_amount(&mut valid_amount),
        "Valid amount should pass"
    );

    let mut empty_amount = "".to_string();
    assert!(
        !validate_amount(&mut empty_amount),
        "Empty amount should fail"
    );
}
