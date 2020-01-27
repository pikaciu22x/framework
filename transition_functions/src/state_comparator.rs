use bls::{PublicKey, SecretKey};
use ethereum_types::H256;
use ssz_types::{BitVector, Error as SzzError, FixedVector, VariableList};
use types::{
    beacon_state::*,
    config::{Config, MainnetConfig, MinimalConfig},
    types::{BeaconBlockHeader, Eth1Data, Fork, Validator},
};
use yaml_rust::{yaml::Yaml, YamlEmitter, YamlLoader};

pub fn compare_states<T: Config>(
    st1: &BeaconState<T>, 
    st2: &BeaconState<T>,
) {
    assert_eq!(st1.genesis_time, st2.genesis_time);
    assert_eq!(st1.slot, st2.slot);
    assert_eq!(st1.fork, st2.fork);

    compare_headers(&st1.latest_block_header, &st2.latest_block_header);
    compare_slice_H256(&st1.block_roots[..], &st2.block_roots[..]);
    compare_slice_H256(&st1.state_roots[..], &st2.state_roots[..]);
    compare_slice_H256(&st1.historical_roots[..], &st2.historical_roots[..]);

    compare_eth1_data(&st1.eth1_data, &st2.eth1_data);
    compare_slice_eth1_data(&st1.eth1_data_votes[..], &st2.eth1_data_votes[..]);
    assert_eq!(st1.eth1_deposit_index, st2.eth1_deposit_index);
    
}

fn compare_headers(
    h1: &BeaconBlockHeader, 
    h2: &BeaconBlockHeader,
) {
    assert_eq!(h1.slot, h2.slot);
    assert_eq!(h1.parent_root, h2.parent_root);
    assert_eq!(h1.state_root, h2.state_root);
    assert_eq!(h1.body_root, h2.body_root);
    assert_eq!(h1.signature, h2.signature);
}

fn compare_slice_H256(
    v1: &[H256],
    v2: &[H256],
) {
    assert_eq!(v1.len(), v2.len());
    for (a, b) in v1.iter().zip(v2.iter()) {
        assert_eq!(a, b);
    }
}

fn compare_slice_eth1_data(
    v1: &[Eth1Data],
    v2: &[Eth1Data],
) {
    assert_eq!(v1.len(), v2.len());
    for (a, b) in v1.iter().zip(v2.iter()) {
        compare_eth1_data(a, b);
    }
}

fn compare_slice_validator(
    v1: &[Validator],
    v2: &[Validator],
) {
    assert_eq!(v1.len(), v2.len());
    for (a, b) in v1.iter().zip(v2.iter()) {
        compare_validator(a, b);
    }
}

fn compare_eth1_data(
    d1: &Eth1Data,
    d2: &Eth1Data,
) {
    assert_eq!(d1.block_hash, d2.block_hash);
    assert_eq!(d1.deposit_count, d2.deposit_count);
    assert_eq!(d1.deposit_root, d2.deposit_root);
}

fn compare_validator(
    v1: &Validator,
    v2: &Validator,
) {
    assert_eq!(v1.pubkey, v2.pubkey);
    assert_eq!(v1.withdrawal_credentials, v2.withdrawal_credentials);
    assert_eq!(v1.effective_balance, v2.effective_balance);
    assert_eq!(v1.slashed, v2.slashed);
    assert_eq!(v1.activation_eligibility_epoch, v2.activation_eligibility_epoch);
    assert_eq!(v1.activation_epoch, v2.activation_epoch);
    assert_eq!(v1.exit_epoch, v2.exit_epoch);
    assert_eq!(v1.withdrawable_epoch, v2.withdrawable_epoch);
}