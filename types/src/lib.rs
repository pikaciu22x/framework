// Lints are currently suppressed to prevent merge conflicts in case our contributors fix their code
// on their own. This attribute should be removed in the future.
#![allow(warnings)]

pub mod beacon_state;
pub mod config;
pub mod consts;
pub mod helper_functions_types;
pub mod primitives;
pub mod types;

pub use crate::beacon_state::{Error as BeaconStateError, *};

#[cfg(test)]
mod spec_tests {
    use core::fmt::Debug;

    use serde::de::DeserializeOwned;
    use ssz::{Decode, Encode};
    use test_generator::test_resources;
    use tree_hash::{SignedRoot, TreeHash};

    use crate::{
        beacon_state::BeaconState,
        config::MinimalConfig,
        types::{
            Attestation, AttestationData, AttesterSlashing, BeaconBlock, BeaconBlockBody,
            BeaconBlockHeader, Checkpoint, Deposit, DepositData, Eth1Data, Fork, HistoricalBatch,
            IndexedAttestation, PendingAttestation, ProposerSlashing, Validator, VoluntaryExit,
        },
    };

    // We do not generate tests for `AggregateAndProof` because this crate does not have that yet.

    #[test_resources("eth2.0-spec-tests/tests/minimal/phase0/ssz_static/Attestation/*/*")]
    fn attestation(case_directory: &str) {
        run_self_signed_case::<Attestation<MinimalConfig>>(case_directory);
    }

    #[test_resources("eth2.0-spec-tests/tests/minimal/phase0/ssz_static/AttestationData/*/*")]
    fn attestation_data(case_directory: &str) {
        run_case::<AttestationData>(case_directory);
    }

    #[test_resources("eth2.0-spec-tests/tests/minimal/phase0/ssz_static/AttesterSlashing/*/*")]
    fn attester_slashing(case_directory: &str) {
        run_case::<AttesterSlashing<MinimalConfig>>(case_directory);
    }

    #[test_resources("eth2.0-spec-tests/tests/minimal/phase0/ssz_static/BeaconBlock/*/*")]
    fn beacon_block(case_directory: &str) {
        run_self_signed_case::<BeaconBlock<MinimalConfig>>(case_directory);
    }

    #[test_resources("eth2.0-spec-tests/tests/minimal/phase0/ssz_static/BeaconBlockBody/*/*")]
    fn beacon_block_body(case_directory: &str) {
        run_case::<BeaconBlockBody<MinimalConfig>>(case_directory);
    }

    #[test_resources("eth2.0-spec-tests/tests/minimal/phase0/ssz_static/BeaconBlockHeader/*/*")]
    fn beacon_block_header(case_directory: &str) {
        run_self_signed_case::<BeaconBlockHeader>(case_directory);
    }

    #[test_resources("eth2.0-spec-tests/tests/minimal/phase0/ssz_static/BeaconState/*/*")]
    fn beacon_state(case_directory: &str) {
        run_case::<BeaconState<MinimalConfig>>(case_directory);
    }

    #[test_resources("eth2.0-spec-tests/tests/minimal/phase0/ssz_static/Checkpoint/*/*")]
    fn checkpoint(case_directory: &str) {
        run_case::<Checkpoint>(case_directory);
    }

    #[test_resources("eth2.0-spec-tests/tests/minimal/phase0/ssz_static/Deposit/*/*")]
    fn deposit(case_directory: &str) {
        run_case::<Deposit>(case_directory);
    }

    #[test_resources("eth2.0-spec-tests/tests/minimal/phase0/ssz_static/DepositData/*/*")]
    fn deposit_data(case_directory: &str) {
        run_self_signed_case::<DepositData>(case_directory);
    }

    #[test_resources("eth2.0-spec-tests/tests/minimal/phase0/ssz_static/Eth1Data/*/*")]
    fn eth1_data(case_directory: &str) {
        run_case::<Eth1Data>(case_directory);
    }

    #[test_resources("eth2.0-spec-tests/tests/minimal/phase0/ssz_static/Fork/*/*")]
    fn fork(case_directory: &str) {
        run_case::<Fork>(case_directory);
    }

    #[test_resources("eth2.0-spec-tests/tests/minimal/phase0/ssz_static/HistoricalBatch/*/*")]
    fn historical_batch(case_directory: &str) {
        run_case::<HistoricalBatch<MinimalConfig>>(case_directory);
    }

    #[test_resources("eth2.0-spec-tests/tests/minimal/phase0/ssz_static/IndexedAttestation/*/*")]
    fn indexed_attestation(case_directory: &str) {
        run_self_signed_case::<IndexedAttestation<MinimalConfig>>(case_directory);
    }

    #[test_resources("eth2.0-spec-tests/tests/minimal/phase0/ssz_static/PendingAttestation/*/*")]
    fn pending_attestation(case_directory: &str) {
        run_case::<PendingAttestation<MinimalConfig>>(case_directory);
    }

    #[test_resources("eth2.0-spec-tests/tests/minimal/phase0/ssz_static/ProposerSlashing/*/*")]
    fn proposer_slashing(case_directory: &str) {
        run_case::<ProposerSlashing>(case_directory);
    }

    #[test_resources("eth2.0-spec-tests/tests/minimal/phase0/ssz_static/Validator/*/*")]
    fn validator(case_directory: &str) {
        run_case::<Validator>(case_directory);
    }

    #[test_resources("eth2.0-spec-tests/tests/minimal/phase0/ssz_static/VoluntaryExit/*/*")]
    fn voluntary_exit(case_directory: &str) {
        run_self_signed_case::<VoluntaryExit>(case_directory);
    }

    fn run_self_signed_case<D>(case_directory: &str)
    where
        D: PartialEq + Debug + DeserializeOwned + Decode + Encode + TreeHash + SignedRoot,
    {
        let signing_root = spec_test_utils::signing_root(case_directory);

        let yaml_value = run_case::<D>(case_directory);

        assert_eq!(yaml_value.signed_root(), signing_root.as_bytes());
    }

    fn run_case<D>(case_directory: &str) -> D
    where
        D: PartialEq + Debug + DeserializeOwned + Decode + Encode + TreeHash,
    {
        let ssz_bytes = spec_test_utils::serialized(case_directory);
        let yaml_value = spec_test_utils::value(case_directory);
        let hash_tree_root = spec_test_utils::hash_tree_root(case_directory);

        let ssz_value = D::from_ssz_bytes(ssz_bytes.as_slice())
            .expect("the file should contain a value encoded in SSZ");

        assert_eq!(ssz_value, yaml_value);
        assert_eq!(ssz_bytes, yaml_value.as_ssz_bytes());
        assert_eq!(yaml_value.tree_hash_root(), hash_tree_root.as_bytes());

        yaml_value
    }
}
