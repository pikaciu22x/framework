use core::consts::ExpConst;
use helper_functions::{
    beacon_state_accessors::{
        get_current_epoch, get_randao_mix, get_total_active_balance, get_validator_churn_limit,
    },
    beacon_state_mutators::{decrease_balance, initiate_validator_exit},
    crypto::hash_tree_root,
    misc::compute_activation_exit_epoch,
    predicates::is_active_validator,
};
use itertools::{Either, Itertools};
use ssz_types::VariableList;
use std::{cmp, mem};
use types::consts::*;
use types::primitives::Gwei;
use types::primitives::*;
use types::types::HistoricalBatch;
use types::{beacon_state::*, config::Config, types::Validator};

fn process_registry_updates<T: Config + ExpConst>(state: &mut BeaconState<T>) {
    let state_copy = state.clone();

    let is_eligible = |validator: &Validator| {
        validator.activation_eligibility_epoch == T::far_future_epoch()
            && validator.effective_balance == T::max_effective_balance()
    };

    let is_exiting_validator = |validator: &Validator| {
        is_active_validator(validator, get_current_epoch(&state_copy))
            && validator.effective_balance <= T::ejection_balance()
    };

    let (eligible, exiting): (Vec<_>, Vec<_>) = state
        .validators
        .iter_mut()
        .enumerate()
        .filter(|(_, validator)| is_eligible(validator) || is_exiting_validator(validator))
        .partition_map(|(i, validator)| {
            if is_eligible(validator) {
                Either::Left(i)
            } else {
                Either::Right(i)
            }
        });

    for index in eligible {
        state.validators[index].activation_eligibility_epoch = get_current_epoch(&state_copy);
    }
    for index in exiting {
        initiate_validator_exit(state, index as u64).unwrap();
    }

    // Queue validators eligible for activation and not dequeued for activation prior to finalized epoch
    let activation_queue = state
        .validators
        .iter()
        .enumerate()
        .filter(|(_, validator)| {
            validator.activation_eligibility_epoch != T::far_future_epoch()
                && validator.activation_epoch
                    >= compute_activation_exit_epoch::<T>(state.finalized_checkpoint.epoch)
        })
        .sorted_by_key(|(_, validator)| validator.activation_eligibility_epoch)
        .map(|(i, _)| i)
        .collect_vec();
    // Dequeued validators for activation up to churn limit (without resetting activation epoch)

    let churn_limit = get_validator_churn_limit(&state);
    let delayed_activation_epoch =
        compute_activation_exit_epoch::<T>(get_current_epoch(&state) as u64);
    for index in activation_queue.into_iter().take(churn_limit as usize) {
        let validator = &mut state.validators[index];
        if validator.activation_epoch == T::far_future_epoch() {
            validator.activation_epoch = delayed_activation_epoch;
        }
    }
}

fn process_slashings<T: Config + ExpConst>(state: &mut BeaconState<T>) {
    let epoch = get_current_epoch(state);
    let total_balance = get_total_active_balance(state).unwrap();

    for validator in state.validators.iter_mut() {
        if validator.slashed
            && epoch + T::epochs_per_slashings_vector() / 2 == validator.withdrawable_epoch
        {
            let increment = T::effective_balance_increment();
            let slashings_sum = state.slashings.iter().sum::<u64>();
            let penalty_numerator = validator.effective_balance / increment
                * cmp::min(slashings_sum * 3, total_balance);
            let penalty = penalty_numerator / total_balance * increment;
            decrease_balance(validator, penalty).unwrap();
        }
    }
}

fn process_final_updates<T: Config + ExpConst>(state: &mut BeaconState<T>) {
    let current_epoch = get_current_epoch(&state);
    let next_epoch = current_epoch + 1 as Epoch;
    //# Reset eth1 data votes
    if (state.slot + 1) % (SLOTS_PER_ETH1_VOTING_PERIOD as u64) == 0 {
        state.eth1_data_votes = VariableList::from(vec![]);
    }
    //# Update effective balances with hysteresis
    for (index, validator) in state.validators.iter_mut().enumerate() {
        let balance = state.balances[index];
        let half_increment = T::effective_balance_increment() / 2;
        if balance < validator.effective_balance
            || validator.effective_balance + 3 * half_increment < balance
        {
            validator.effective_balance = cmp::min(
                balance - balance % T::effective_balance_increment(),
                T::max_effective_balance(),
            );
        }
    }

    //# Reset slashings
    state.slashings[(next_epoch % T::epochs_per_slashings_vector()) as usize] = 0 as Gwei;
    //# Set randao mix
    state.randao_mixes[(next_epoch % EPOCHS_PER_HISTORICAL_VECTOR) as usize] =
        get_randao_mix(&state, current_epoch).unwrap();
    //# Set historical root accumulator
    if next_epoch % (T::slots_per_historical_root() / T::slots_per_epoch()) == 0 {
        let historical_batch = HistoricalBatch::<T> {
            block_roots: state.block_roots.clone(),
            state_roots: state.state_roots.clone(),
        };
        state
            .historical_roots
            .push(hash_tree_root(&historical_batch))
            .unwrap();
    }
    //# Rotate current/previous epoch attestations
    state.previous_epoch_attestations = mem::replace(
        &mut state.current_epoch_attestations,
        VariableList::from(vec![]),
    );
}

#[cfg(test)]
mod process_epoch_tests {
    #[test]
    fn process_good_epoch() {
        assert_eq!(1, 1);
    }
}
