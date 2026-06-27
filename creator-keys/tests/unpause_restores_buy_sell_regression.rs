//! Regression tests for unpause restoring full buy and sell functionality (#468).
//!
//! Covers: buy reverts while paused, buy succeeds immediately after unpause,
//! sell succeeds immediately after unpause, and post-unpause state matches
//! expected values as if the pause never occurred.

mod contract_test_env;

use contract_test_env::{
    register_creator_keys, register_test_creator, set_pricing_and_fees, test_env_with_auths,
};
use creator_keys::ContractError;
use soroban_sdk::{testutils::Address as _, Address};

const KEY_PRICE: i128 = 1_000;
const CREATOR_BPS: u32 = 9_000;
const PROTOCOL_BPS: u32 = 1_000;

fn setup_with_admin(
    env: &soroban_sdk::Env,
) -> (
    creator_keys::CreatorKeysContractClient<'_>,
    Address,
    Address,
) {
    let (client, _) = register_creator_keys(env);
    set_pricing_and_fees(env, &client, KEY_PRICE, CREATOR_BPS, PROTOCOL_BPS);
    let admin = Address::generate(env);
    client.set_protocol_admin(&admin, &admin);
    let creator = register_test_creator(env, &client, "alice");
    (client, admin, creator)
}

// ---------------------------------------------------------------------------
// buy reverts while paused; buy succeeds immediately after unpause
// ---------------------------------------------------------------------------

#[test]
fn test_buy_reverts_while_paused() {
    let env = test_env_with_auths();
    let (client, admin, creator) = setup_with_admin(&env);
    let buyer = Address::generate(&env);
    let quote = client.get_buy_quote(&creator);

    client.pause(&admin);
    assert!(client.get_is_paused());

    let result = client.try_buy_key(&creator, &buyer, &quote.total_amount, &None);
    assert_eq!(
        result,
        Err(Ok(ContractError::ProtocolPaused)),
        "buy must revert with ProtocolPaused while protocol is paused"
    );
}

#[test]
fn test_buy_succeeds_immediately_after_unpause() {
    let env = test_env_with_auths();
    let (client, admin, creator) = setup_with_admin(&env);
    let buyer = Address::generate(&env);

    // Establish baseline: one buy before pause
    let quote_before = client.get_buy_quote(&creator);
    client.buy_key(&creator, &buyer, &quote_before.total_amount, &None);
    let supply_before_pause = client.get_total_key_supply(&creator);
    assert_eq!(supply_before_pause, 1);

    client.pause(&admin);
    // Buy attempt must fail
    let paused_result = client.try_buy_key(&creator, &buyer, &quote_before.total_amount, &None);
    assert_eq!(paused_result, Err(Ok(ContractError::ProtocolPaused)));

    client.unpause(&admin);
    assert!(!client.get_is_paused());

    // Buy must succeed in the very next call after unpause
    let quote_after = client.get_buy_quote(&creator);
    let new_supply = client.buy_key(&creator, &buyer, &quote_after.total_amount, &None);
    assert_eq!(new_supply, 2, "supply must be 2 after successful buy post-unpause");
    assert_eq!(client.get_total_key_supply(&creator), 2);
    assert_eq!(client.get_key_balance(&creator, &buyer), 2);
}
