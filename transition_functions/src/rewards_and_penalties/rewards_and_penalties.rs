use crate::attestations::attestations::AttestableBlock;
use core::consts::ExpConst;
use helper_functions::{
    beacon_state_accessors::{
        get_attesting_indices, get_current_epoch, get_previous_epoch, get_total_active_balance,
        get_total_balance,
    },
    beacon_state_mutators::{decrease_balance, increase_balance},
    math::integer_squareroot,
    predicates::is_active_validator,
};
use types::{
    beacon_state::BeaconState,
    config::Config,
    primitives::{Gwei, ValidatorIndex},
};

pub trait StakeholderBlock<T>
where
    T: Config + ExpConst,
{
    fn get_base_reward(&self, index: ValidatorIndex) -> Gwei;
    fn get_attestation_deltas(&self) -> (Vec<Gwei>, Vec<Gwei>);
    fn process_rewards_and_penalties(&mut self);
}

impl<T> StakeholderBlock<T> for BeaconState<T>
where
    T: Config + ExpConst,
{
    fn get_base_reward(&self, index: ValidatorIndex) -> Gwei {
        let total_balance = get_total_active_balance(self).unwrap();
        let effective_balance = self.validators[index as usize].effective_balance;
        return (effective_balance * T::base_reward_factor()
            / integer_squareroot(total_balance)
            / T::base_rewards_per_epoch()) as Gwei;
    }

    fn get_attestation_deltas(&self) -> (Vec<Gwei>, Vec<Gwei>) {
        let previous_epoch = get_previous_epoch(self);
        let total_balance = get_total_active_balance(self).unwrap();
        let mut rewards = Vec::new();
        let mut penalties = Vec::new();
        let mut eligible_validator_indices: Vec<ValidatorIndex> = Vec::new();

        for (index, v) in self.validators.iter().enumerate() {
            rewards.push(0 as Gwei);
            penalties.push(0 as Gwei);
            if is_active_validator(v, previous_epoch)
                || (v.slashed && previous_epoch + 1 < v.withdrawable_epoch)
            {
                eligible_validator_indices.push(index as ValidatorIndex);
            }
        }
        //# Micro-incentives for matching FFG source, FFG target, and head
        let matching_source_attestations = self.get_matching_source_attestations(previous_epoch);
        let matching_target_attestations = self.get_matching_target_attestations(previous_epoch);
        let matching_head_attestations = self.get_matching_head_attestations(previous_epoch);
        let vec = vec![
            matching_source_attestations.clone(),
            matching_target_attestations.clone(),
            matching_head_attestations.clone(),
        ];

        for attestations in vec.into_iter() {
            let unslashed_attesting_indices = self.get_unslashed_attesting_indices(attestations);
            let attesting_balance = get_total_balance(self, &unslashed_attesting_indices).unwrap();

            for index in eligible_validator_indices.iter() {
                if unslashed_attesting_indices.contains(&index) {
                    rewards[*index as usize] += ((self.get_base_reward(*index) * attesting_balance)
                        / total_balance)
                        as ValidatorIndex;
                } else {
                    penalties[*index as usize] += self.get_base_reward(*index);
                }
            }
        }

        //# Proposer and inclusion delay micro-rewards
        for index in self
            .get_unslashed_attesting_indices(matching_source_attestations.clone())
            .iter()
        {
            let attestation = matching_source_attestations
                .iter()
                .fold(None, |min, x| match min {
                    None => Some(x),
                    Some(y) => Some(
                        if get_attesting_indices(self, &x.data, &x.aggregation_bits)
                            .unwrap()
                            .contains(index)
                            && x.inclusion_delay < y.inclusion_delay
                        {
                            x
                        } else {
                            y
                        },
                    ),
                })
                .unwrap();

            let proposer_reward =
                (self.get_base_reward(*index) / T::proposer_reward_quotient()) as Gwei;
            rewards[attestation.proposer_index as usize] += proposer_reward;
            let max_attester_reward = self.get_base_reward(*index) - proposer_reward;
            rewards[*index as usize] += (max_attester_reward / attestation.inclusion_delay) as Gwei;
        }
        //# Inactivity penalty
        let finality_delay = previous_epoch - self.finalized_checkpoint.epoch;
        if finality_delay > T::min_epochs_to_inactivity_penalty() {
            let matching_target_attesting_indices =
                self.get_unslashed_attesting_indices(matching_target_attestations);
            for index in eligible_validator_indices.iter() {
                penalties[*index as usize] +=
                    (T::base_rewards_per_epoch() * self.get_base_reward(*index)) as Gwei;
                if !(matching_target_attesting_indices.contains(index)) {
                    penalties[*index as usize] +=
                        ((self.validators[*index as usize].effective_balance * finality_delay)
                            / T::inactivity_penalty_quotient()) as Gwei;
                }
            }
        }
        return (rewards, penalties);
    }

    fn process_rewards_and_penalties(&mut self) {
        if get_current_epoch(self) == T::genesis_epoch() {
            return;
        }
        let (rewards, penalties) = self.get_attestation_deltas();
        for (index, validator) in self.validators.clone().iter_mut().enumerate() {
            increase_balance(self, index as u64, rewards[index]).unwrap();
            decrease_balance(self, index as u64, penalties[index]).unwrap();
        }
    }
}

#[cfg(test)]
mod process_slot_tests {
    use types::types::Validator;
    use types::{beacon_state::*, config::MainnetConfig};

    // use crate::{config::*};
    use super::*;
    #[test]
    fn test_base_reward() {
        assert_eq!(1, 1);
        let mut bs: BeaconState<MainnetConfig> = BeaconState {
            ..BeaconState::default()
        };
        let mut val: Validator = Validator {
            ..Validator::default()
        };
        val.effective_balance = 5;
        val.slashed = false;
        bs.validators.push(val).unwrap();
        let index = 0;
        assert_eq!(5 * 64 / 4, bs.get_base_reward(index));
    }

    fn test_get_attestation_deltas() {
        let mut bs: BeaconState<MainnetConfig> = BeaconState {
            slot: 1,
            ..BeaconState::default()
        };
        let mut val: Validator = Validator {
            ..Validator::default()
        };
        val.effective_balance = 5;
        let mut val2: Validator = Validator {
            ..Validator::default()
        };
        val.effective_balance = 5;
        val2.effective_balance = 4;
        val.slashed = false;
        val2.slashed = true;
        bs.validators.push(val).unwrap();
        bs.validators.push(val2).unwrap();
        // let base_reward1 = 5*64/4;
        // let base_reward2 = 4*64/4;

        // let reward1 = base_reward1;
        // let penalty1 = 0;
        // let reward2= 0;
        // let penalty2 = base_reward2;
        // bs.process_rewards_and_penalties();
    }
}
