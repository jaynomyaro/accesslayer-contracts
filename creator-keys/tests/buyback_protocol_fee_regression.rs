//! Regression tests for protocol fee applied during creator buyback (#469).
//!
//! Covers: treasury balance increases by the correct protocol fee on buyback,
//! creator is charged bonding curve price plus protocol fee, and creator fee
//! recipient balance is unchanged (creator fee is waived on buyback).

mod contract_test_env;

use contract_test_env::{
    compute_expected_protocol_fee, register_creator_keys, register_test_creator,
    set_pricing_and_fees, test_env_with_auths,
};

const KEY_PRICE: i128 = 1_000;
const CREATOR_BPS: u32 = 9_000;
const PROTOCOL_BPS: u32 = 1_000;
