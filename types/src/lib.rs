pub mod beacon_state;
pub mod config;
pub mod consts;
pub mod helper_functions_types;
pub mod primitives;
pub mod types;

pub use crate::beacon_state::{Error as BeaconStateError, *};

// #[cfg(test)]
// mod spec_tests {
//     use core::fmt::Debug;

//     use serde::de::DeserializeOwned;
//     use ssz_new::{SszDecode, SszEncode};
//     use test_generator::test_resources;
//     use tree_hash::TreeHash;

//     use crate::config::{MainnetConfig, MinimalConfig};

//     mod tested_types {
//         pub use crate::{beacon_state::BeaconState, types::*};
//     }

//     macro_rules! tests_for_type {
//         (
//             $type: ident $(<_ $bracket: tt)?,
//             $mainnet_glob: literal,
//             $minimal_glob: literal,
//         ) => {
//             mod $type {
//                 use super::*;

//                 #[test_resources($mainnet_glob)]
//                 fn mainnet(case_directory: &str) {
//                     run_case::<tested_types::$type$(<MainnetConfig $bracket)?>(case_directory);
//                 }

//                 #[test_resources($minimal_glob)]
//                 fn minimal(case_directory: &str) {
//                     run_case::<tested_types::$type$(<MinimalConfig $bracket)?>(case_directory);
//                 }
//             }
//         };
//     }

//     We do not generate tests for `AggregateAndProof` and `Eth1Block`
//     because this crate does not have those yet.

//     tests_for_type! {
//         run_self_signed_case::<Attestation<_>>,
//         "eth2.0-spec-tests/tests/mainnet/phase0/ssz_static/Attestation/*/*",
//         "eth2.0-spec-tests/tests/minimal/phase0/ssz_static/Attestation/*/*",
//     }

//     tests_for_type! {
//         run_case::<AttestationData>,
//         "eth2.0-spec-tests/tests/mainnet/phase0/ssz_static/AttestationData/*/*",
//         "eth2.0-spec-tests/tests/minimal/phase0/ssz_static/AttestationData/*/*",
//     }

//     tests_for_type! {
//         run_case::<AttesterSlashing<_>>,
//         "eth2.0-spec-tests/tests/mainnet/phase0/ssz_static/AttesterSlashing/*/*",
//         "eth2.0-spec-tests/tests/minimal/phase0/ssz_static/AttesterSlashing/*/*",
//     }

//     tests_for_type! {
//         run_self_signed_case::<BeaconBlock<_>>,
//         "eth2.0-spec-tests/tests/mainnet/phase0/ssz_static/BeaconBlock/*/*",
//         "eth2.0-spec-tests/tests/minimal/phase0/ssz_static/BeaconBlock/*/*",
//     }

//     tests_for_type! {
//         run_case::<BeaconBlockBody<_>>,
//         "eth2.0-spec-tests/tests/mainnet/phase0/ssz_static/BeaconBlockBody/*/*",
//         "eth2.0-spec-tests/tests/minimal/phase0/ssz_static/BeaconBlockBody/*/*",
//     }

//     tests_for_type! {
//         run_self_signed_case::<BeaconBlockHeader>,
//         "eth2.0-spec-tests/tests/mainnet/phase0/ssz_static/BeaconBlockHeader/*/*",
//         "eth2.0-spec-tests/tests/minimal/phase0/ssz_static/BeaconBlockHeader/*/*",
//     }

//     tests_for_type! {
//         run_case::<BeaconState<_>>,
//         "eth2.0-spec-tests/tests/mainnet/phase0/ssz_static/BeaconState/*/*",
//         "eth2.0-spec-tests/tests/minimal/phase0/ssz_static/BeaconState/*/*",
//     }

//     tests_for_type! {
//         run_case::<Checkpoint>,
//         "eth2.0-spec-tests/tests/mainnet/phase0/ssz_static/Checkpoint/*/*",
//         "eth2.0-spec-tests/tests/minimal/phase0/ssz_static/Checkpoint/*/*",
//     }

//     tests_for_type! {
//         run_case::<Deposit>,
//         "eth2.0-spec-tests/tests/mainnet/phase0/ssz_static/Deposit/*/*",
//         "eth2.0-spec-tests/tests/minimal/phase0/ssz_static/Deposit/*/*",
//     }

//     tests_for_type! {
//         run_self_signed_case::<DepositData>,
//         "eth2.0-spec-tests/tests/mainnet/phase0/ssz_static/DepositData/*/*",
//         "eth2.0-spec-tests/tests/minimal/phase0/ssz_static/DepositData/*/*",
//     }

//     tests_for_type! {
//         run_case::<Eth1Data>,
//         "eth2.0-spec-tests/tests/mainnet/phase0/ssz_static/Eth1Data/*/*",
//         "eth2.0-spec-tests/tests/minimal/phase0/ssz_static/Eth1Data/*/*",
//     }

//     tests_for_type! {
//         run_case::<Fork>,
//         "eth2.0-spec-tests/tests/mainnet/phase0/ssz_static/Fork/*/*",
//         "eth2.0-spec-tests/tests/minimal/phase0/ssz_static/Fork/*/*",
//     }

//     tests_for_type! {
//         run_case::<HistoricalBatch<_>>,
//         "eth2.0-spec-tests/tests/mainnet/phase0/ssz_static/HistoricalBatch/*/*",
//         "eth2.0-spec-tests/tests/minimal/phase0/ssz_static/HistoricalBatch/*/*",
//     }

//     tests_for_type! {
//         run_self_signed_case::<IndexedAttestation<_>>,
//         "eth2.0-spec-tests/tests/mainnet/phase0/ssz_static/IndexedAttestation/*/*",
//         "eth2.0-spec-tests/tests/minimal/phase0/ssz_static/IndexedAttestation/*/*",
//     }

//     tests_for_type! {
//         run_case::<PendingAttestation<_>>,
//         "eth2.0-spec-tests/tests/mainnet/phase0/ssz_static/PendingAttestation/*/*",
//         "eth2.0-spec-tests/tests/minimal/phase0/ssz_static/PendingAttestation/*/*",
//     }

//     tests_for_type! {
//         run_case::<ProposerSlashing>,
//         "eth2.0-spec-tests/tests/mainnet/phase0/ssz_static/ProposerSlashing/*/*",
//         "eth2.0-spec-tests/tests/minimal/phase0/ssz_static/ProposerSlashing/*/*",
//     }

//     tests_for_type! {
//         run_case::<Validator>,
//         "eth2.0-spec-tests/tests/mainnet/phase0/ssz_static/Validator/*/*",
//         "eth2.0-spec-tests/tests/minimal/phase0/ssz_static/Validator/*/*",
//     }

//     tests_for_type! {
//         run_self_signed_case::<VoluntaryExit>,
//         "eth2.0-spec-tests/tests/mainnet/phase0/ssz_static/VoluntaryExit/*/*",
//         "eth2.0-spec-tests/tests/minimal/phase0/ssz_static/VoluntaryExit/*/*",
//     }

//     tests_for_type! {
//         SignedVoluntaryExit,
//         "eth2.0-spec-tests/tests/mainnet/phase0/ssz_static/SignedVoluntaryExit/*/*",
//         "eth2.0-spec-tests/tests/minimal/phase0/ssz_static/SignedVoluntaryExit/*/*",
//     }

//     tests_for_type! {
//         SigningRoot,
//         "eth2.0-spec-tests/tests/mainnet/phase0/ssz_static/SigningRoot/*/*",
//         "eth2.0-spec-tests/tests/minimal/phase0/ssz_static/SigningRoot/*/*",
//     }

//     tests_for_type! {
//         Validator,
//         "eth2.0-spec-tests/tests/mainnet/phase0/ssz_static/Validator/*/*",
//         "eth2.0-spec-tests/tests/minimal/phase0/ssz_static/Validator/*/*",
//     }

//     tests_for_type! {
//         VoluntaryExit,
//         "eth2.0-spec-tests/tests/mainnet/phase0/ssz_static/VoluntaryExit/*/*",
//         "eth2.0-spec-tests/tests/minimal/phase0/ssz_static/VoluntaryExit/*/*",
//     }

//     fn run_case<D>(case_directory: &str) -> D
//     where
//         D: PartialEq + Debug + DeserializeOwned + SszDecode + SszEncode + TreeHash,
//     {
//         let ssz_bytes = spec_test_utils::serialized(case_directory);
//         let yaml_value = spec_test_utils::value(case_directory);
//         let hash_tree_root = spec_test_utils::hash_tree_root(case_directory);

//         let ssz_value = D::from_ssz_bytes(ssz_bytes.as_slice())
//             .expect("the file should contain a value encoded in SSZ");

//         assert_eq!(ssz_value, yaml_value);
//         assert_eq!(ssz_bytes, yaml_value.as_ssz_bytes());
//         assert_eq!(yaml_value.tree_hash_root(), hash_tree_root.as_bytes());

//         yaml_value
//     }
// }
