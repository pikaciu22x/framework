use ring::digest::{digest, SHA256};
use ssz_types::{BitList, VariableList};
use std::cmp;
use std::collections::BTreeSet;
use std::convert::TryFrom;
use typenum::marker_traits::Unsigned;
use types::{beacon_state::BeaconState, config::Config, primitives::*, types::*};

use crate::{
    error::Error,
    math::{int_to_bytes, int_to_bytes_32},
    misc::*,
    predicates::is_active_validator,
};

pub fn get_current_epoch<C: Config>(state: &BeaconState<C>) -> Epoch {
    compute_epoch_at_slot::<C>(state.slot)
}

pub fn get_previous_epoch<C: Config>(state: &BeaconState<C>) -> Epoch {
    let current_epoch = get_current_epoch(state);
    let genesis_epoch = C::genesis_epoch();

    if current_epoch > genesis_epoch {
        current_epoch - 1
    } else {
        genesis_epoch
    }
}

pub fn get_block_root<C: Config>(state: &BeaconState<C>, epoch: Epoch) -> Result<H256, Error> {
    // todo: change to compute start slot of epoch when implemented
    get_block_root_at_slot(state, epoch * C::SlotsPerEpoch::to_u64())
}

pub fn get_block_root_at_slot<C: Config>(
    state: &BeaconState<C>,
    slot: Slot,
) -> Result<H256, Error> {
    if !(slot < state.slot && state.slot <= slot + C::SlotsPerHistoricalRoot::to_u64()) {
        return Err(Error::SlotOutOfRange);
    }

    match usize::try_from(slot % C::SlotsPerHistoricalRoot::to_u64()) {
        Err(_err) => Err(Error::IndexOutOfRange),
        Ok(id) => Ok(state.block_roots[id]),
    }
}

pub fn get_randao_mix<C: Config>(state: &BeaconState<C>, epoch: Epoch) -> Result<H256, Error> {
    match usize::try_from(epoch) {
        Err(_err) => Err(Error::IndexOutOfRange),
        Ok(id) => Ok(state.randao_mixes[id % C::EpochsPerHistoricalVector::to_usize()]),
    }
}

pub fn get_active_validator_indices<C: Config>(
    state: &BeaconState<C>,
    epoch: Epoch,
) -> Vec<ValidatorIndex> {
    let mut active_validator_indices = Vec::new();
    for (i, v) in state.validators.iter().enumerate() {
        if is_active_validator(v, epoch) {
            active_validator_indices.push(i as u64);
        }
    }
    active_validator_indices
}

pub fn get_validator_churn_limit<C: Config>(state: &BeaconState<C>) -> Result<u64, Error> {
    let active_validator_indices = get_active_validator_indices(state, get_current_epoch(state));

    Ok(cmp::max(
        C::min_per_epoch_churn_limit(),
        active_validator_indices.len() as u64 / C::churn_limit_quotient(),
    ))
}

pub fn get_seed<C: Config>(
    state: &BeaconState<C>,
    epoch: Epoch,
    domain_type: DomainType,
) -> Result<H256, Error> {
    let mix = get_randao_mix::<C>(
        state,
        epoch + C::EpochsPerHistoricalVector::to_u64() - C::min_seed_lookahead() - 1,
    )?;

    let mut seed: [u8; 44] = [0; 44];
    seed[0..4].copy_from_slice(&int_to_bytes_32(domain_type, 4));
    seed[4..12].copy_from_slice(&int_to_bytes(epoch, 8));
    seed[12..44].copy_from_slice(&mix[..]);

    let mut hash_bytes: [u8; 32] = [0; 32];
    hash_bytes[0..32].copy_from_slice(digest(&SHA256, &seed).as_ref());

    Ok(H256::from(hash_bytes))
}

pub fn get_committee_count_at_slot<C: Config>(
    state: &BeaconState<C>,
    slot: Slot,
) -> Result<u64, Error> {
    let epoch = compute_epoch_at_slot::<C>(slot);

    let committees_per_slot = cmp::min(
        C::ShardCount::to_u64() / C::SlotsPerEpoch::to_u64(),
        get_active_validator_indices(state, epoch).len() as u64,
    );

    Ok(cmp::max(1, committees_per_slot) * C::SlotsPerEpoch::to_u64())
}

pub fn get_committee_count<C: Config>(state: &BeaconState<C>, epoch: Epoch) -> Result<u64, Error> {
    let committees_per_slot = cmp::min(
        C::ShardCount::to_u64() / C::SlotsPerEpoch::to_u64(),
        get_active_validator_indices(state, epoch).len() as u64,
    );

    Ok(cmp::max(1, committees_per_slot) * C::SlotsPerEpoch::to_u64())
}

pub fn get_beacon_committee<C: Config>(
    state: &BeaconState<C>,
    slot: Slot,
    index: CommitteeIndex,
) -> Result<Vec<ValidatorIndex>, Error> {
    let epoch = compute_epoch_at_slot::<C>(slot);
    let committees_per_slot = get_committee_count_at_slot(state, slot)?;
    compute_committee::<C>(
        &get_active_validator_indices(state, epoch),
        &(get_seed(state, epoch, C::domain_attestation())?),
        (slot % C::SlotsPerEpoch::to_u64()) * committees_per_slot + index,
        committees_per_slot * C::SlotsPerEpoch::to_u64(),
    )
}

pub fn get_total_balance<C: Config>(
    state: &BeaconState<C>,
    indices: &[ValidatorIndex],
) -> Result<u64, Error> {
    let mut sum = 0;
    for (_i, index) in indices.iter().enumerate() {
        match usize::try_from(*index) {
            Err(_err) => return Err(Error::IndexOutOfRange),
            Ok(id) => sum += state.validators[id].effective_balance,
        }
    }
    Ok(sum)
}

pub fn get_total_active_balance<C: Config>(state: &BeaconState<C>) -> Result<u64, Error> {
    get_total_balance::<C>(
        state,
        &get_active_validator_indices::<C>(state, get_current_epoch::<C>(state)),
    )
}

pub fn get_domain<C: Config>(
    state: &BeaconState<C>,
    domain_type: DomainType,
    message_epoch: Option<Epoch>,
) -> Domain {
    let epoch = message_epoch.unwrap_or_else(|| get_current_epoch(state));
    let fork_version = if epoch < state.fork.epoch {
        &state.fork.previous_version
    } else {
        &state.fork.current_version
    };
    compute_domain::<C>(domain_type, Some(fork_version))
}

pub fn get_indexed_attestation<C: Config>(
    state: &BeaconState<C>,
    attestation: &Attestation<C>,
) -> Result<IndexedAttestation<C>, Error> {
    let attesting_indices =
        get_attesting_indices(state, &attestation.data, &attestation.aggregation_bits)?;

    let custody_bit_1_indices =
        get_attesting_indices(state, &attestation.data, &attestation.custody_bits)?;

    let custody_bit_0_indices = &attesting_indices - &custody_bit_1_indices;

    let custody_bit_0_indices_list = match VariableList::new(
        custody_bit_0_indices
            .into_iter()
            .map(|x| x as u64)
            .collect(),
    ) {
        Err(_err) => return Err(Error::ConversionToVariableList),
        Ok(list) => list,
    };

    let custody_bit_1_indices_list = match VariableList::new(
        custody_bit_1_indices
            .into_iter()
            .map(|x| x as u64)
            .collect(),
    ) {
        Err(_err) => return Err(Error::ConversionToVariableList),
        Ok(list) => list,
    };

    Ok(IndexedAttestation {
        custody_bit_0_indices: custody_bit_0_indices_list,
        custody_bit_1_indices: custody_bit_1_indices_list,
        data: attestation.data.clone(),
        signature: attestation.signature.clone(),
    })
}

pub fn get_attesting_indices<C: Config>(
    state: &BeaconState<C>,
    data: &AttestationData,
    bits: &BitList<C::MaxValidatorsPerCommittee>,
) -> Result<BTreeSet<ValidatorIndex>, Error> {
    let committee = get_beacon_committee(state, data.slot, data.index)?;
    println!("{length}", length = committee.len());
    println!("{length}", length = bits.len());
    if bits.len() != committee.len() {
        return Err(Error::AttestationBitsInvalid);
    }
    Ok(committee
        .iter()
        .enumerate()
        .filter_map(|(i, index)| match bits.get(i) {
            Ok(true) => Some(*index),
            _ => None,
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ssz_types::{FixedVector, VariableList};
    use types::config::MainnetConfig;
    use types::types::{Fork, Validator};

    #[test]
    fn test_get_current_epoch() {
        let bs: BeaconState<MainnetConfig> = BeaconState {
            slot: 9,
            ..BeaconState::default()
        };
        assert_eq!(get_current_epoch::<MainnetConfig>(&bs), 1);
    }

    #[test]
    fn test_get_previous_epoch() {
        let bs: BeaconState<MainnetConfig> = BeaconState {
            slot: 17,
            ..BeaconState::default()
        };
        assert_eq!(get_previous_epoch(&bs), 1);
    }

    #[test]
    fn test_get_previous_epoch_genesis() {
        let bs: BeaconState<MainnetConfig> = BeaconState {
            slot: 0,
            ..BeaconState::default()
        };
        assert_eq!(get_previous_epoch(&bs), MainnetConfig::genesis_epoch());
    }

    #[test]
    fn test_get_block_root() {
        let mut block_roots_vec = Vec::new();

        for x in 0..32 {
            block_roots_vec.push(H256::from([x; 32]));
        }

        let bs: BeaconState<MainnetConfig> = BeaconState {
            slot: 32,
            block_roots: FixedVector::from(block_roots_vec),
            ..BeaconState::default()
        };

        assert_eq!(get_block_root(&bs, 3), Ok(H256::from([24; 32])));
    }

    #[test]
    fn test_get_seed() {
        let bs: BeaconState<MainnetConfig> = BeaconState {
            randao_mixes: FixedVector::from(vec![
                H256::from([1; 32]),
                H256::from([2; 32]),
            ]),
            ..BeaconState::default()
        };

        let actual = get_seed::<MainnetConfig>(&bs, 1, 1_u32);

        let expected = H256::from([
            0x14, 0x81, 0x4a, 0x14, 0x7c, 0x51, 0x6b, 0x2a, 0xc3, 0xda, 0xe0, 0x72,
            0xea, 0xf9, 0xd5, 0xca, 0x2e, 0x3a, 0xbd, 0xca, 0x96, 0x96, 0xd2, 0x44,
            0x31, 0x3c, 0x35, 0x12, 0x99, 0x33, 0xe3, 0x36,
        ]);

        assert_eq!(actual, Ok(expected));
    }

    #[test]
    fn test_get_block_root_at_slot() {
        let bs: BeaconState<MainnetConfig> = BeaconState {
            slot: 2,
            block_roots: FixedVector::from(vec![H256::from([0; 32]), H256::from([1; 32])]),
            ..BeaconState::default()
        };
        assert_eq!(get_block_root_at_slot(&bs, 1), Ok(H256::from([1; 32])));
    }

    #[test]
    fn test_get_block_root_at_slot_slot_equals_beacon_state_slot() {
        let bs: BeaconState<MainnetConfig> = BeaconState {
            slot: 0,
            ..BeaconState::default()
        };
        assert_eq!(
            get_block_root_at_slot(&bs, 0).err(),
            Some(Error::SlotOutOfRange),
        );
    }

    #[test]
    fn test_get_randao_mix() {
        let bs: BeaconState<MainnetConfig> = BeaconState {
            randao_mixes: FixedVector::from(vec![
                H256::from([5; 32]),
                H256::from([5; 32]),
                H256::from([5; 32]),
            ]),
            ..BeaconState::default()
        };

        assert_eq!(get_randao_mix(&bs, 2), Ok(H256::from([5; 32])))
    }

    #[test]
    fn test_get_active_validator_indices() {
        let v1 = Validator {
            activation_epoch: 1,
            exit_epoch: 2,
            ..Validator::default()
        };
        let v2 = Validator {
            activation_epoch: 0,
            exit_epoch: 1,
            ..Validator::default()
        };
        let bs: BeaconState<MainnetConfig> = BeaconState {
            validators: VariableList::from(vec![v1, v2]),
            ..BeaconState::default()
        };
        assert_eq!(get_active_validator_indices(&bs, 0), vec![1]);
    }

    #[test]
    fn test_get_validator_churn_limit() {
        let v1 = Validator {
            effective_balance: 11,
            activation_epoch: 0,
            exit_epoch: 2,
            ..Validator::default()
        };
        let bs: BeaconState<MainnetConfig> = BeaconState {
            validators: VariableList::from(vec![v1]),
            ..BeaconState::default()
        };

        assert_eq!(
            get_validator_churn_limit(&bs),
            Ok(MainnetConfig::min_per_epoch_churn_limit())
        )
    }

    #[test]
    fn test_get_committee_count() {
        let v1 = Validator {
            effective_balance: 11,
            activation_epoch: 0,
            exit_epoch: 2,
            ..Validator::default()
        };
        let bs: BeaconState<MainnetConfig> = BeaconState {
            validators: VariableList::from(vec![v1]),
            ..BeaconState::default()
        };

        assert_eq!(
            get_committee_count(&bs, 0_u64),
            Ok(<MainnetConfig as Config>::ShardCount::to_u64())
        )
    }

    #[test]
    fn test_get_active_balance() {
        let v1 = Validator {
            effective_balance: 11,
            activation_epoch: 0,
            exit_epoch: 2,
            ..Validator::default()
        };
        let v2 = Validator {
            effective_balance: 7,
            activation_epoch: 0,
            exit_epoch: 1,
            ..Validator::default()
        };
        let v3 = Validator {
            effective_balance: 5,
            activation_epoch: 0,
            exit_epoch: 1,
            ..Validator::default()
        };
        let bs: BeaconState<MainnetConfig> = BeaconState {
            validators: VariableList::from(vec![v1, v2, v3]),
            ..BeaconState::default()
        };

        assert_eq!(get_total_balance(&bs, &[0, 2]), Ok(16_u64))
    }

    #[test]
    fn test_get_total_active_balance() {
        let v1 = Validator {
            effective_balance: 10,
            activation_epoch: 0,
            exit_epoch: 2,
            ..Validator::default()
        };
        let v2 = Validator {
            effective_balance: 2,
            activation_epoch: 0,
            exit_epoch: 1,
            ..Validator::default()
        };
        let bs: BeaconState<MainnetConfig> = BeaconState {
            validators: VariableList::from(vec![v1, v2]),
            ..BeaconState::default()
        };

        assert_eq!(get_total_active_balance(&bs), Ok(12_u64))
    }

    #[test]
    fn test_get_domain_previous_version() {
        let bs: BeaconState<MainnetConfig> = BeaconState {
            fork: Fork {
                previous_version: [0_u8, 0_u8, 0_u8, 1_u8],
                current_version: [0_u8, 0_u8, 1_u8, 0_u8],
                epoch: 2,
            },
            ..BeaconState::default()
        };
        let domain_type: DomainType = 2_u32;
        let expected: u64 = 0x0100_0000_0000_0002_u64;

        assert_eq!(
            get_domain::<MainnetConfig>(&bs, domain_type, Some(1)),
            expected
        );
    }

    #[test]
    fn test_get_domain_current_version() {
        let bs: BeaconState<MainnetConfig> = BeaconState {
            fork: Fork {
                previous_version: [0_u8, 0_u8, 0_u8, 1_u8],
                current_version: [0_u8, 0_u8, 1_u8, 0_u8],
                epoch: 1,
            },
            ..BeaconState::default()
        };
        let domain_type: DomainType = 2_u32;
        let expected: u64 = 0x0001_0000_0000_0002_u64;

        assert_eq!(
            get_domain::<MainnetConfig>(&bs, domain_type, Some(1)),
            expected
        );
    }

    #[test]
    fn test_get_domain_default_version() {
        let bs: BeaconState<MainnetConfig> = BeaconState {
            slot: 9,
            fork: Fork {
                previous_version: [0_u8, 0_u8, 0_u8, 1_u8],
                current_version: [0_u8, 0_u8, 1_u8, 0_u8],
                epoch: 2,
            },
            ..BeaconState::default()
        };
        let domain_type: DomainType = 2_u32;
        let expected: u64 = 0x0100_0000_0000_0002_u64;

        assert_eq!(
            get_domain::<MainnetConfig>(&bs, domain_type, None),
            expected
        );
    }

    #[test]
    fn test_get_indexed_attestation() {
        let validator = Validator {
            activation_epoch: 0,
            exit_epoch: u64::max_value(),
            ..Validator::default()
        };
        
        let bs: BeaconState<MainnetConfig> = BeaconState {
            validators: VariableList::from(vec![validator; 64]),
            randao_mixes: FixedVector::from(vec![H256::from([5; 32]); 64]),
            ..BeaconState::<MainnetConfig>::default()
        };

        let aggregation_bits = BitList::with_capacity(64).expect("BitList creation failed");
        let custody_bits = BitList::with_capacity(64).expect("BitList creation failed");

        let attestation: Attestation<MainnetConfig> = Attestation {
            aggregation_bits: aggregation_bits,
            data: AttestationData::default(),
            custody_bits: custody_bits,
            signature: Signature::default(),
        };

        let expected: IndexedAttestation<MainnetConfig> = IndexedAttestation::default();
        let actual = get_indexed_attestation(&bs, &attestation);

        assert_eq!(actual, Ok(expected));
    }
}
