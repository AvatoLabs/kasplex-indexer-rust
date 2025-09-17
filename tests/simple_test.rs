use kaspa_indexer_rust::operations::*;
use kaspa_indexer_rust::storage::types::*;

#[test]
fn test_deploy_operation() {
    let mut script = DataScriptType {
        p: "KRC-20".to_string(),
        op: "KRC-20".to_string(),
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
        mod_type: "deploy".to_string(),
        name: None,
        ca: None,
    };

    let result = DeployOperation::validate(&mut script, "test_tx_id", 110165001, false);
    assert!(result, "Deploy operation validation should pass");
    assert_eq!(script.p, "KRC-20");
    assert_eq!(script.op, "KRC-20");
    assert_eq!(script.mod_type, "");
}

#[test]
fn test_token_state_creation() {
    let token = StateTokenType {
        tick: "TEST".to_string(),
        max: "1000000".to_string(),
        lim: "1000".to_string(),
        pre: "0".to_string(),
        dec: 8,
        mod_type: "0".to_string(),
        from: "kaspa:test_address".to_string(),
        to: "kaspa:test_address".to_string(),
        minted: "0".to_string(),
        burned: "0".to_string(),
        name: "Test Token".to_string(),
        tx_id: "test_tx_id".to_string(),
        op_add: 0,
        op_mod: 0,
        mts_add: 0,
        mts_mod: 0,
    };

    assert_eq!(token.tick, "TEST");
    assert_eq!(token.max, "1000000");
    assert_eq!(token.mod_type, "0");
}
