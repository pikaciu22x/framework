use types::{
    beacon_state::*,
    config::Config,
    primitives::H256,
    types::{BeaconBlockHeader, Eth1Data, PendingAttestation, Validator},
};

pub fn compare_states<T: Config>(st1: &BeaconState<T>, st2: &BeaconState<T>) {
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

    compare_slice_validator(&st1.validators[..], &st2.validators[..]);
    compare_slice_u64(&st1.balances[..], &st2.balances[..]);

    compare_slice_H256(&st1.randao_mixes[..], &st2.randao_mixes[..]);

    compare_slice_u64(&st1.slashings[..], &st2.slashings[..]);

    compare_slice_pending_attestation(
        &st1.previous_epoch_attestations[..],
        &st2.previous_epoch_attestations[..],
    );
    compare_slice_pending_attestation(
        &st1.current_epoch_attestations[..],
        &st2.current_epoch_attestations[..],
    );

    assert_eq!(st1.justification_bits, st2.justification_bits);
    assert_eq!(
        st1.previous_justified_checkpoint,
        st2.previous_justified_checkpoint
    );
    assert_eq!(
        st1.current_justified_checkpoint,
        st2.current_justified_checkpoint
    );
    assert_eq!(st1.finalized_checkpoint, st2.finalized_checkpoint);
}

fn compare_headers(h1: &BeaconBlockHeader, h2: &BeaconBlockHeader) {
    assert_eq!(h1.slot, h2.slot);
    assert_eq!(h1.parent_root, h2.parent_root);
    assert_eq!(h1.state_root, h2.state_root);
    assert_eq!(h1.body_root, h2.body_root);
    assert_eq!(h1.signature, h2.signature);
}

fn compare_slice_H256(v1: &[H256], v2: &[H256]) {
    assert_eq!(v1.len(), v2.len());
    for (a, b) in v1.iter().zip(v2.iter()) {
        assert_eq!(a, b);
    }
}

fn compare_slice_eth1_data(v1: &[Eth1Data], v2: &[Eth1Data]) {
    assert_eq!(v1.len(), v2.len());
    for (a, b) in v1.iter().zip(v2.iter()) {
        compare_eth1_data(a, b);
    }
}

fn compare_slice_validator(v1: &[Validator], v2: &[Validator]) {
    assert_eq!(v1.len(), v2.len());
    for (a, b) in v1.iter().zip(v2.iter()) {
        compare_validator(a, b);
    }
}

fn compare_slice_u64(v1: &[u64], v2: &[u64]) {
    assert_eq!(v1.len(), v2.len());
    for (a, b) in v1.iter().zip(v2.iter()) {
        assert_eq!(a, b);
    }
}

fn compare_slice_pending_attestation<T: Config>(
    v1: &[PendingAttestation<T>],
    v2: &[PendingAttestation<T>],
) {
    assert_eq!(v1.len(), v2.len());
    for (a, b) in v1.iter().zip(v2.iter()) {
        assert_eq!(a, b);
    }
}

fn compare_eth1_data(d1: &Eth1Data, d2: &Eth1Data) {
    assert_eq!(d1.block_hash, d2.block_hash);
    assert_eq!(d1.deposit_count, d2.deposit_count);
    assert_eq!(d1.deposit_root, d2.deposit_root);
}

fn compare_validator(v1: &Validator, v2: &Validator) {
    assert_eq!(v1.pubkey, v2.pubkey);
    assert_eq!(v1.withdrawal_credentials, v2.withdrawal_credentials);
    assert_eq!(v1.effective_balance, v2.effective_balance);
    assert_eq!(v1.slashed, v2.slashed);
    assert_eq!(
        v1.activation_eligibility_epoch,
        v2.activation_eligibility_epoch
    );
    assert_eq!(v1.activation_epoch, v2.activation_epoch);
    assert_eq!(v1.exit_epoch, v2.exit_epoch);
    assert_eq!(v1.withdrawable_epoch, v2.withdrawable_epoch);
}

// fn compadre_pending_attestations<T: Config>(
//     a1: &PendingAttestation<T>,
//     a2: &PendingAttestation<T>,
// ) {
//     assert_eq!(a1.aggregation_bits, a2.aggregation_bits);
//     assert_eq!(a1, a2);
// }
