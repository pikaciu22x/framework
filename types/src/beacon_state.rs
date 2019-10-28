use serde::{Deserialize, Serialize};
use ssz_derive::{Decode, Encode};
use ssz_types::{BitVector, FixedVector, VariableList};
use std::cmp;
use std::convert::TryFrom;
use tree_hash_derive::TreeHash;
use typenum::marker_traits::Unsigned;

use crate::{config::*, consts, error::Error, primitives::*, types::*};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, Encode, Decode, TreeHash, Default)]
pub struct BeaconState<C: Config> {
    pub genesis_time: u64,
    pub slot: Slot,
    pub fork: Fork,

    // History
    pub latest_block_header: BeaconBlockHeader,
    pub block_roots: FixedVector<H256, C::SlotsPerHistoricalRoot>,
    pub state_roots: FixedVector<H256, C::SlotsPerHistoricalRoot>,
    pub historical_roots: VariableList<H256, C::HistoricalRootsLimit>,

    // Eth1 Data
    pub eth1_data: Eth1Data,
    pub eth1_data_votes: VariableList<Eth1Data, C::SlotsPerEth1VotingPeriod>,
    pub eth1_deposit_index: u64,

    // Registry
    pub validators: VariableList<Validator, C::ValidatorRegistryLimit>,
    pub balances: VariableList<u64, C::ValidatorRegistryLimit>,

    // Shuffling
    pub start_shard: u64,
    pub randao_mixes: FixedVector<H256, C::EpochsPerHistoricalVector>,
    pub active_index_roots: FixedVector<H256, C::EpochsPerHistoricalVector>,
    pub compact_committees_roots: FixedVector<H256, C::EpochsPerHistoricalVector>,

    // Slashings
    pub slashings: FixedVector<u64, C::EpochsPerSlashingsVector>,

    // Attestations
    pub previous_epoch_attestations:
        VariableList<PendingAttestation<C>, C::MaxAttestationsPerEpoch>,
    pub current_epoch_attestations: VariableList<PendingAttestation<C>, C::MaxAttestationsPerEpoch>,

    // Crosslinks
    pub previous_crosslinks: FixedVector<Crosslink, C::ShardCount>,
    pub current_crosslinks: FixedVector<Crosslink, C::ShardCount>,

    // Finality
    pub justification_bits: BitVector<consts::JustificationBitsLength>,
    pub previous_justified_checkpoint: Checkpoint,
    pub current_justified_checkpoint: Checkpoint,
    pub finalized_checkpoint: Checkpoint,
}

impl<C: Config> BeaconState<C> {
    pub fn compute_activation_exit_epoch(&self, epoch: Epoch) -> Epoch {
        epoch + 1 + C::activation_exit_delay()
    }

    pub fn get_block_root_at_slot(&self, slot: Slot) -> Result<H256, Error> {
        if !(slot < self.slot && self.slot <= slot + C::SlotsPerHistoricalRoot::to_u64()) {
            return Err(Error::SlotOutOfRange);
        }

        match usize::try_from(slot % C::SlotsPerHistoricalRoot::to_u64()) {
            Err(_err) => Err(Error::IndexOutOfRange),
            Ok(id) => Ok(self.block_roots[id]),
        }
    }

    pub fn get_block_root(&self, epoch: Epoch) -> Result<H256, Error> {
        // todo: change to compute start slot of epoch when implemented
        self.get_block_root_at_slot(epoch * C::SlotsPerEpoch::to_u64())
    }

    pub fn get_active_validator_indices(&self, epoch: Epoch) -> Vec<ValidatorIndex> {
        let mut active_validator_indices = Vec::new();
        for (i, v) in self.validators.iter().enumerate() {
            if v.is_active_validator(epoch) {
                active_validator_indices.push(i as u64);
            }
        }
        active_validator_indices
    }

    pub fn increase_balance(&mut self, index: ValidatorIndex, delta: Gwei) {
        match usize::try_from(index) {
            Err(_err) => {}
            Ok(id) => self.balances[id] += delta,
        }
    }

    pub fn decrease_balance(&mut self, index: ValidatorIndex, delta: Gwei) {
        match usize::try_from(index) {
            Err(_err) => {}
            Ok(id) => {
                self.balances[id] = if delta > self.balances[id] {
                    0
                } else {
                    self.balances[id] - delta
                }
            }
        }
    }

    pub fn get_current_epoch(&self) -> Epoch {
        self.slot / C::SlotsPerEpoch::to_u64()
    }

    pub fn get_previous_epoch(&self) -> Epoch {
        let current_epoch = self.get_current_epoch();
        let genesis_epoch = C::genesis_epoch();

        if current_epoch > genesis_epoch {
            current_epoch - 1
        } else {
            genesis_epoch
        }
    }

    pub fn get_randao_mix(&self, epoch: Epoch) -> Result<H256, Error> {
        match usize::try_from(epoch) {
            Err(_err) => Err(Error::IndexOutOfRange),
            Ok(id) => Ok(self.randao_mixes[id % C::EpochsPerHistoricalVector::to_usize()]),
        }
    }

    pub fn get_validator_churn_limit(&self) -> Result<u64, Error> {
        let active_validator_indices = self.get_active_validator_indices(self.get_current_epoch());

        Ok(cmp::max(
            C::min_per_epoch_churn_limit(),
            active_validator_indices.len() as u64 / C::churn_limit_quotient(),
        ))
    }

    pub fn get_committee_count(&self, epoch: Epoch) -> Result<u64, Error> {
        let committees_per_slot = cmp::min(
            C::ShardCount::to_u64() / C::SlotsPerEpoch::to_u64(),
            self.get_active_validator_indices(epoch).len() as u64,
        );

        Ok(cmp::max(1, committees_per_slot) * C::SlotsPerEpoch::to_u64())
    }

    pub fn get_total_balance(&self, indices: &[ValidatorIndex]) -> Result<u64, Error> {
        let mut sum = 0;
        for (_i, index) in indices.iter().enumerate() {
            match usize::try_from(*index) {
                Err(_err) => return Err(Error::IndexOutOfRange),
                Ok(id) => sum += self.validators[id].effective_balance,
            }
        }
        Ok(sum)
    }

    pub fn get_total_active_balance(&self) -> Result<u64, Error> {
        self.get_total_balance(&self.get_active_validator_indices(self.get_current_epoch()))
    }

    pub fn compute_start_slot_of_epoch(&self, epoch: Epoch) -> Slot {
        epoch * C::SlotsPerEpoch::to_u64()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_activation_exit_epoch() {
        let bs: BeaconState<MainnetConfig> = BeaconState::default();
        assert_eq!(bs.compute_activation_exit_epoch(0), 5);
    }

    #[test]
    fn get_block_root_at_slot() {
        let bs: BeaconState<MainnetConfig> = BeaconState {
            slot: 2,
            block_roots: FixedVector::from(vec![H256::from([0; 32]), H256::from([1; 32])]),
            ..BeaconState::default()
        };
        assert_eq!(bs.get_block_root_at_slot(1), Ok(H256::from([1; 32])));
    }

    #[test]
    fn get_block_root_at_slot_slot_equals_beacon_state_slot() {
        let bs: BeaconState<MainnetConfig> = BeaconState {
            slot: 0,
            ..BeaconState::default()
        };
        assert_eq!(
            bs.get_block_root_at_slot(0).err(),
            Some(Error::SlotOutOfRange),
        );
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

        assert_eq!(bs.get_block_root(3), Ok(H256::from([24; 32])));
    }

    #[test]
    fn get_active_validator_indices() {
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
        assert_eq!(bs.get_active_validator_indices(0), vec![1]);
    }

    #[test]
    fn increase_balance() {
        let mut bs: BeaconState<MainnetConfig> = BeaconState {
            balances: VariableList::from(vec![0]),
            ..BeaconState::default()
        };
        bs.increase_balance(0, 1);
        assert_eq!(bs.balances[0], 1);
    }

    #[test]
    fn test_decrease_balance() {
        let mut bs: BeaconState<MainnetConfig> = BeaconState {
            balances: VariableList::from(vec![5]),
            ..BeaconState::default()
        };
        bs.decrease_balance(0, 3);
        assert_eq!(bs.balances[0], 2);
    }

    #[test]
    fn test_decrease_balance_to_negative() {
        let mut bs: BeaconState<MainnetConfig> = BeaconState {
            balances: VariableList::from(vec![0]),
            ..BeaconState::default()
        };
        bs.decrease_balance(0, 1);
        assert_eq!(bs.balances[0], 0);
    }

    #[test]
    fn get_current_epoch() {
        let bs: BeaconState<MainnetConfig> = BeaconState {
            slot: 9,
            ..BeaconState::default()
        };
        assert_eq!(bs.get_current_epoch(), 1);
    }

    #[test]
    fn test_get_previous_epoch() {
        let bs: BeaconState<MainnetConfig> = BeaconState {
            slot: 17,
            ..BeaconState::default()
        };
        assert_eq!(bs.get_previous_epoch(), 1);
    }

    #[test]
    fn test_get_previous_epoch_genesis() {
        let bs: BeaconState<MainnetConfig> = BeaconState {
            slot: 0,
            ..BeaconState::default()
        };
        assert_eq!(bs.get_previous_epoch(), MainnetConfig::genesis_epoch());
    }

    #[test]
    fn test_compute_start_slot_of_epoch() {
        let bs: BeaconState<MainnetConfig> = BeaconState {
            slot: 0,
            ..BeaconState::default()
        };
        assert_eq!(
            bs.compute_start_slot_of_epoch(10_u64),
            <MainnetConfig as Config>::SlotsPerEpoch::to_u64() * 10_u64
        )
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

        assert_eq!(bs.get_total_active_balance(), Ok(12_u64))
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

        assert_eq!(bs.get_total_balance(&[0, 2]), Ok(16_u64))
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
            bs.get_committee_count(0_u64),
            Ok(<MainnetConfig as Config>::ShardCount::to_u64())
        )
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
            bs.get_validator_churn_limit(),
            Ok(MainnetConfig::min_per_epoch_churn_limit())
        )
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

        assert_eq!(bs.get_randao_mix(2), Ok(H256::from([5; 32])))
    }
}
