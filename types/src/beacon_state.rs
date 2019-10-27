use serde::{Deserialize, Serialize};
use ssz_derive::{Decode, Encode};
use ssz_types::{BitVector, FixedVector, VariableList};
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
        Ok(self.block_roots[(slot % C::SlotsPerHistoricalRoot::to_u64()) as usize])
    }

    pub fn get_block_root(&self, epoch: Epoch) -> Result<H256, Error> {
        // todo: change to compute start slot of epoch when implemented
        self.get_block_root_at_slot(Slot::from(epoch * C::SlotsPerEpoch::to_u64()))
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
        self.balances[index as usize] += delta
    }

    pub fn decrease_balance(&mut self, index: ValidatorIndex, delta: Gwei) {
        self.balances[index as usize] = if delta > self.balances[index as usize] {
            0
        } else {
            self.balances[index as usize] - delta
        }
    }

    pub fn get_current_epoch(&self) -> Epoch {
        Epoch::from(self.slot / C::SlotsPerEpoch::to_u64())
    }

    pub fn get_previous_epoch(&self) -> Epoch {
        let current_epoch = self.get_current_epoch();
        let genesis_epoch = C::genesis_epoch();

        if current_epoch > genesis_epoch {
            Epoch::from(current_epoch - 1)
        } else {
            genesis_epoch
        }
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
        assert_eq!(bs.get_block_root_at_slot(0).err(), Some(Error::SlotOutOfRange));
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
}
