use std::cmp;
use std::convert::TryFrom;
use typenum::marker_traits::Unsigned;
use types::{beacon_state::BeaconState, config::Config, primitives::*};

use crate::{error::Error, misc::epoch_of_slot, predicates::is_active_validator};

pub fn get_current_epoch<C: Config>(state: &BeaconState<C>) -> Epoch {
    epoch_of_slot::<C>(state.slot)
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

pub fn get_committee_count<C: Config>(state: &BeaconState<C>, epoch: Epoch) -> Result<u64, Error> {
    let committees_per_slot = cmp::min(
        C::ShardCount::to_u64() / C::SlotsPerEpoch::to_u64(),
        get_active_validator_indices(state, epoch).len() as u64,
    );

    Ok(cmp::max(1, committees_per_slot) * C::SlotsPerEpoch::to_u64())
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

#[cfg(test)]
mod tests {
    use super::*;
    use ssz_types::{FixedVector, VariableList};
    use types::config::MainnetConfig;
    use types::types::Validator;

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
}
