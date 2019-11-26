use crate::crypto::{bls_aggregate_pubkeys, bls_verify, hash, hash_tree_root};
use crate::error::Error;
use std::convert::TryFrom;
use typenum::marker_traits::Unsigned;
use types::{
    beacon_state::BeaconState,
    config::Config,
    primitives::*,
    types::{AttestationData, IndexedAttestation, Validator},
};

pub fn is_slashable_validator(validator: &Validator, epoch: Epoch) -> bool {
    !validator.slashed
        && validator.activation_epoch <= epoch
        && epoch < validator.withdrawable_epoch
}

pub fn is_active_validator(validator: &Validator, epoch: Epoch) -> bool {
    validator.activation_epoch <= epoch && epoch < validator.exit_epoch
}

pub fn is_slashable_attestation_data(data_1: &AttestationData, data_2: &AttestationData) -> bool {
    // Double vote
    (data_1 != data_2 && data_1.target.epoch == data_2.target.epoch) ||
    // Surround vote
    (data_1.source.epoch < data_2.source.epoch && data_2.target.epoch < data_1.target.epoch)
}

pub fn validate_indexed_attestation<C: Config>(
    state: &BeaconState<C>,
    indexed_attestation: &IndexedAttestation<C>,
) -> Result<(), Error> {
    let indices = &indexed_attestation.attesting_indices;

    // Verify max number of indices
    if !(indices.len() < C::MaxValidatorsPerCommittee::to_usize()) {
        return Err(Error::MaxIndicesExceeded);
    }

    // Verify indices are sorted
    let is_sorted = indices.windows(2).all(|w| w[0] <= w[1]);
    if !is_sorted {
        return Err(Error::BadValidatorIndicesOrdering);
    }

    // let pubkeys = state
    //     .validators
    //     .into_iter()
    //     .enumerate()
    //     .filter_map(|(i, v)| {
    //         if indices.contains(&i) {
    //             None
    //         } else {
    //             Some(v.pubkey)
    //         }
    //     });

    // if !bls_verify(
    //     bls_aggregate_pubkeys(pubkeys),
    //     message_hash=hash_tree_root(indexed_attestation.data),
    //     signature=indexed_attestation.signature,
    //     domain=get_domain(state, DOMAIN_BEACON_ATTESTER, indexed_attestation.data.target.epoch),
    // ) {

    // }

    Ok(())
}

pub fn is_valid_merkle_branch<C: Config>(
    leaf: &H256,
    branch: &[H256],
    depth: u64,
    index: u64,
    root: &H256,
) -> Result<bool, Error> {
    let mut value: H256 = *leaf;

    match usize::try_from(depth) {
        Ok(depth_usize) => {
            for (i, node) in branch.iter().enumerate().take(depth_usize) {
                if index / (2 ^ (i as u64)) % 2 == 0 {
                    value = H256::from_slice(&hash(&join_hashes(&value, node)));
                } else {
                    value = H256::from_slice(&hash(&join_hashes(node, &value)));
                }
            }
            Ok(value == *root)
        }
        Err(_) => Err(Error::IndexOutOfRange),
    }
}

fn join_hashes<'a>(hash1: &'a H256, hash2: &H256) -> Vec<u8> {
    hash1
        .as_ref()
        .iter()
        .chain(hash2.as_ref())
        .copied()
        .collect::<Vec<u8>>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ssz_types::VariableList;
    use types::config::MainnetConfig;
    use types::types::Checkpoint;

    #[test]
    fn test_is_slashable_validator() {
        let v = Validator {
            slashed: false,
            activation_epoch: 0,
            withdrawable_epoch: 1,
            ..Validator::default()
        };
        assert_eq!(is_slashable_validator(&v, 0), true);
    }

    #[test]
    fn test_is_slashable_validator_already_slashed() {
        let v = Validator {
            slashed: true,
            activation_epoch: 0,
            withdrawable_epoch: 1,
            ..Validator::default()
        };
        assert_eq!(is_slashable_validator(&v, 0), false);
    }

    #[test]
    fn test_is_slashable_validator_activation_epoch_greater_than_epoch() {
        let v = Validator {
            slashed: false,
            activation_epoch: 1,
            withdrawable_epoch: 2,
            ..Validator::default()
        };
        assert_eq!(is_slashable_validator(&v, 0), false);
    }

    #[test]
    fn test_is_slashable_validator_withdrawable_epoch_equals_epoch() {
        let v = Validator {
            slashed: false,
            activation_epoch: 0,
            withdrawable_epoch: 1,
            ..Validator::default()
        };
        assert_eq!(is_slashable_validator(&v, 1), false);
    }

    #[test]
    fn test_is_active_validator() {
        let v = Validator {
            activation_epoch: 0,
            exit_epoch: 1,
            ..Validator::default()
        };
        assert_eq!(is_active_validator(&v, 0), true);
    }

    #[test]
    fn test_is_active_validator_activation_epoch_greater_than_epoch() {
        let v = Validator {
            activation_epoch: 1,
            exit_epoch: 2,
            ..Validator::default()
        };
        assert_eq!(is_active_validator(&v, 0), false);
    }

    #[test]
    fn test_is_active_validator_exit_epoch_equals_epoch() {
        let v = Validator {
            activation_epoch: 0,
            exit_epoch: 1,
            ..Validator::default()
        };
        assert_eq!(is_active_validator(&v, 1), false);
    }

    #[test]
    fn test_is_slashable_attestation_data_double_vote_false() {
        let attestation_data_1 = AttestationData {
            target: Checkpoint {
                epoch: 1,
                root: H256::from([0; 32]),
            },
            ..AttestationData::default()
        };
        let attestation_data_2 = AttestationData {
            target: Checkpoint {
                epoch: 1,
                root: H256::from([0; 32]),
            },
            ..AttestationData::default()
        };
        assert_eq!(
            is_slashable_attestation_data(&attestation_data_1, &attestation_data_2),
            false
        );
    }

    #[test]
    fn test_is_slashable_attestation_data_double_vote_true() {
        let attestation_data_1 = AttestationData {
            target: Checkpoint {
                epoch: 1,
                root: H256::from([0; 32]),
            },
            ..AttestationData::default()
        };
        let attestation_data_2 = AttestationData {
            target: Checkpoint {
                epoch: 1,
                root: H256::from([1; 32]),
            },
            ..AttestationData::default()
        };
        assert_eq!(
            is_slashable_attestation_data(&attestation_data_1, &attestation_data_2),
            true
        );
    }

    #[test]
    fn test_is_slashable_attestation_data_surround_vote_true() {
        let attestation_data_1 = AttestationData {
            source: Checkpoint {
                epoch: 0,
                root: H256::from([0; 32]),
            },
            target: Checkpoint {
                epoch: 3,
                root: H256::from([0; 32]),
            },
            ..AttestationData::default()
        };
        let attestation_data_2 = AttestationData {
            source: Checkpoint {
                epoch: 1,
                root: H256::from([1; 32]),
            },
            target: Checkpoint {
                epoch: 2,
                root: H256::from([0; 32]),
            },
            ..AttestationData::default()
        };
        assert_eq!(
            is_slashable_attestation_data(&attestation_data_1, &attestation_data_2),
            true
        );
    }

    // #[test]
    // fn test_is_valid_indexed_attestation_max_indices_exceeded() {
    //     let state: BeaconState<MainnetConfig> = BeaconState::<MainnetConfig>::default();
    //     let bit_0_indices: Vec<u64> = (0_u64..4096_u64).collect();
    //     let bit_1_indices: Vec<u64> = vec![1];
    //     let attestation: IndexedAttestation<MainnetConfig> = IndexedAttestation {
    //         custody_bit_0_indices: VariableList::from(bit_0_indices),
    //         custody_bit_1_indices: VariableList::from(bit_1_indices),
    //         ..IndexedAttestation::default()
    //     };
    //     assert_eq!(
    //         is_valid_indexed_attestation::<MainnetConfig>(&state, &attestation),
    //         Err(Error::MaxIndicesExceeded)
    //     );
    // }

    // #[test]
    // fn test_is_valid_indexed_attestation_bad_validator_indices_ordering() {
    //     let state: BeaconState<MainnetConfig> = BeaconState::<MainnetConfig>::default();
    //     let bit_0_indices: Vec<u64> = (0_u64..64_u64).collect();
    //     let bit_1_indices: Vec<u64> = vec![66_u64, 65_u64];
    //     let attestation: IndexedAttestation<MainnetConfig> = IndexedAttestation {
    //         custody_bit_0_indices: VariableList::from(bit_0_indices),
    //         custody_bit_1_indices: VariableList::from(bit_1_indices),
    //         ..IndexedAttestation::default()
    //     };
    //     assert_eq!(
    //         is_valid_indexed_attestation::<MainnetConfig>(&state, &attestation),
    //         Err(Error::BadValidatorIndicesOrdering)
    //     );
    // }
}
