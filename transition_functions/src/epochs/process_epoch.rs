use crate::attestations::AttestableBlock;
use crate::rewards_and_penalties::StakeholderBlock;
use helper_functions::{
    beacon_state_accessors::{
        get_block_root, get_current_epoch, get_previous_epoch, get_randao_mix,
        get_total_active_balance, get_validator_churn_limit,
    },
    beacon_state_mutators::*,
    crypto::hash_tree_root,
    misc::compute_activation_exit_epoch,
    predicates::is_active_validator,
};
use itertools::{Either, Itertools};
use ssz_types::VariableList;
use std::cmp;
use std::convert::TryFrom;
use typenum::Unsigned as _;
use types::consts::*;
use types::{
    beacon_state::{BeaconState, Error},
    config::Config,
    types::{Checkpoint, HistoricalBatch, Validator},
};

pub fn process_epoch<T: Config>(state: &mut BeaconState<T>) {
    process_justification_and_finalization(state)
        .expect("Error during justification and finalization");
    process_rewards_and_penalties(state).expect("Error durng rewards and penalties");
    process_registry_updates(state);
    process_slashings(state);
    process_final_updates(state);
}

fn process_justification_and_finalization<T: Config>(
    state: &mut BeaconState<T>,
) -> Result<(), Error> {
    if get_current_epoch(state) <= GENESIS_EPOCH + 1 {
        return Ok(());
    }

    let previous_epoch = get_previous_epoch(state);
    let current_epoch = get_current_epoch(state);
    let old_previous_justified_checkpoint = state.previous_justified_checkpoint;
    let old_current_justified_checkpoint = state.current_justified_checkpoint;

    // Process justifications
    state.previous_justified_checkpoint = state.current_justified_checkpoint;
    state.justification_bits.shift_up(1)?;
    // Previous epoch
    let matching_target_attestations = state.get_matching_target_attestations(previous_epoch);
    if state.get_attesting_balance(matching_target_attestations) * 3
        >= get_total_active_balance(state)? * 2
    {
        state.current_justified_checkpoint = Checkpoint {
            epoch: previous_epoch,
            root: get_block_root(state, previous_epoch)?,
        };
        state.justification_bits.set(1, true)?;
    }

    // Current epoch
    let matching_target_attestations = state.get_matching_target_attestations(current_epoch);
    if state.get_attesting_balance(matching_target_attestations) * 3
        >= get_total_active_balance(state)? * 2
    {
        state.current_justified_checkpoint = Checkpoint {
            epoch: current_epoch,
            root: get_block_root(state, current_epoch)?,
        };
        state.justification_bits.set(0, true)?;
    }

    // The 2nd/3rd/4th most recent epochs are all justified, the 2nd using the 4th as source
    // or
    // The 2nd/3rd most recent epochs are both justified, the 2nd using the 3rd as source
    // or
    // The 1st/2nd/3rd most recent epochs are all justified, the 1st using the 3nd as source
    // or
    // The 1st/2nd most recent epochs are both justified, the 1st using the 2nd as source
    if ((1..4).all(|i| state.justification_bits.get(i).unwrap_or(false))
        && old_previous_justified_checkpoint.epoch + 3 == current_epoch)
        || ((1..3).all(|i| state.justification_bits.get(i).unwrap_or(false))
            && old_previous_justified_checkpoint.epoch + 2 == current_epoch)
        || ((0..3).all(|i| state.justification_bits.get(i).unwrap_or(false))
            && old_current_justified_checkpoint.epoch + 2 == current_epoch)
        || ((0..2).all(|i| state.justification_bits.get(i).unwrap_or(false))
            && old_current_justified_checkpoint.epoch + 1 == current_epoch)
    {
        state.finalized_checkpoint = old_current_justified_checkpoint;
    }
    Ok(())
}

fn process_registry_updates<T: Config>(state: &mut BeaconState<T>) {
    let state_copy = state.clone();

    let is_eligible = |validator: &Validator| {
        validator.activation_eligibility_epoch == FAR_FUTURE_EPOCH
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
        initiate_validator_exit(state, index as u64).expect("validator exit error");
    }

    // Queue validators eligible for activation and not dequeued for activation prior to finalized epoch
    let activation_queue = state
        .validators
        .iter()
        .enumerate()
        .filter(|(_, validator)| {
            validator.activation_eligibility_epoch != FAR_FUTURE_EPOCH
                && validator.activation_epoch
                    >= compute_activation_exit_epoch::<T>(state.finalized_checkpoint.epoch)
        })
        .sorted_by_key(|(_, validator)| validator.activation_eligibility_epoch)
        .map(|(i, _)| i)
        .collect_vec();
    // Dequeued validators for activation up to churn limit (without resetting activation epoch)

    let churn_limit = get_validator_churn_limit(state).expect("Validator churn limit error");
    let delayed_activation_epoch = compute_activation_exit_epoch::<T>(get_current_epoch(state));

    for index in activation_queue
        .into_iter()
        .take(usize::try_from(churn_limit).expect("Conversion error"))
    {
        let validator = &mut state.validators[index];
        if validator.activation_epoch == FAR_FUTURE_EPOCH {
            validator.activation_epoch = delayed_activation_epoch;
        }
    }
}

fn process_rewards_and_penalties<T: Config>(state: &mut BeaconState<T>) -> Result<(), Error> {
    if get_current_epoch(state) == GENESIS_EPOCH {
        return Ok(());
    }
    let (rewards, penalties) = state.get_attestation_deltas();
    for (index, _) in state.validators.clone().iter_mut().enumerate() {
        increase_balance(state, index as u64, rewards[index]).expect("Balance error");
        decrease_balance(state, index as u64, penalties[index]).expect("Balance error");
    }
    Ok(())
}

fn process_slashings<T: Config>(state: &mut BeaconState<T>) {
    let epoch = get_current_epoch(state);
    let total_balance = get_total_active_balance(state).expect("Balance error");

    for (index, validator) in state.validators.clone().iter_mut().enumerate() {
        if validator.slashed
            && epoch + T::EpochsPerSlashingsVector::U64 / 2 == validator.withdrawable_epoch
        {
            let increment = T::effective_balance_increment();
            let slashings_sum = state.slashings.iter().sum::<u64>();
            let penalty_numerator = validator.effective_balance / increment
                * cmp::min(slashings_sum * 3, total_balance);
            let penalty = penalty_numerator / total_balance * increment;
            decrease_balance(state, index as u64, penalty).expect("Balance error");
        }
    }
}

fn process_final_updates<T: Config>(state: &mut BeaconState<T>) {
    let current_epoch = get_current_epoch(state);
    let next_epoch = current_epoch + 1;
    //# Reset eth1 data votes
    if (state.slot + 1) % T::SlotsPerEth1VotingPeriod::U64 == 0 {
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
    let index =
        usize::try_from(next_epoch % T::EpochsPerHistoricalVector::U64).expect("Conversion error");
    state.slashings[index] = 0;
    //# Set randao mix
    state.randao_mixes[index] = get_randao_mix(state, current_epoch).expect("Randao error");
    //# Set historical root accumulator
    if next_epoch % (T::SlotsPerHistoricalRoot::U64 / T::SlotsPerEpoch::U64) == 0 {
        let historical_batch = HistoricalBatch::<T> {
            block_roots: state.block_roots.clone(),
            state_roots: state.state_roots.clone(),
        };
        state
            .historical_roots
            .push(hash_tree_root(&historical_batch))
            .expect("Push error");
    }
    //# Rotate current/previous epoch attestations
    state.previous_epoch_attestations = state.current_epoch_attestations.clone();
    state.current_epoch_attestations = VariableList::from(vec![]);
}

// #[cfg(test)]
// mod process_epoch_tests {
//     use super::*;
//     // use mockall::mock;
//     use types::config::MainnetConfig;
//     /*
//     mock! {
//         BeaconState<C: Config + 'static> {}
//         trait BeaconStateAccessor {
//             fn get_current_epoch(&self) -> Epoch;
//             fn get_previous_epoch(&self) -> Epoch;
//             fn get_block_root(&self, _epoch: Epoch) -> Result<H256, hfError>;
//         }
//     */
//     }

// //     //     let mut bs = MockBeaconState::<MainnetConfig>::new();
// //     //     bs.expect_get_current_epoch().return_const(5_u64);
// //     //     assert_eq!(5, bs.get_current_epoch());
// //     // }
// // }

// #[cfg(test)]
// mod spec_tests {
//     use core::fmt::Debug;

//     use test_generator::test_resources;
//     use types::{beacon_state::BeaconState, config::MinimalConfig};
//     use void::Void;

//     use super::*;

//     // We do not honor `bls_setting` in epoch processing tests because none of them customize it.

//     macro_rules! tests_for_sub_transition {
//         (
//             $module_name: ident,
//             $sub_transition: expr,
//             $mainnet_glob: literal,
//             $minimal_glob: literal,
//         ) => {
//             mod $module_name {
//                 use super::*;

//                 #[test_resources($mainnet_glob)]
//                 fn mainnet(case_directory: &str) {
//                     run_case::<MainnetConfig, _, _>(case_directory, $sub_transition);
//                 }

//                 #[test_resources($minimal_glob)]
//                 fn minimal(case_directory: &str) {
//                     run_case::<MinimalConfig, _, _>(case_directory, $sub_transition);
//                 }
//             }
//         };
//     }

//     tests_for_sub_transition! {
//         justification_and_finalization,
//         process_justification_and_finalization,
//         "eth2.0-spec-tests/tests/mainnet/phase0/epoch_processing/justification_and_finalization/*/*",
//         "eth2.0-spec-tests/tests/minimal/phase0/epoch_processing/justification_and_finalization/*/*",
//     }

//     // There are no mainnet test cases for the `rewards_and_penalties` sub-transition.
//     #[test_resources(
//         "eth2.0-spec-tests/tests/minimal/phase0/epoch_processing/rewards_and_penalties/*/*"
//     )]
//     fn minimal_rewards_and_penalties(case_directory: &str) {
//         run_case::<MinimalConfig, _, _>(case_directory, process_rewards_and_penalties);
//     }

//     tests_for_sub_transition! {
//         registry_updates,
//         wrap_in_ok(process_registry_updates),
//         "eth2.0-spec-tests/tests/mainnet/phase0/epoch_processing/registry_updates/*/*",
//         "eth2.0-spec-tests/tests/minimal/phase0/epoch_processing/registry_updates/*/*",
//     }

//     tests_for_sub_transition! {
//         slashings,
//         wrap_in_ok(process_slashings),
//         "eth2.0-spec-tests/tests/mainnet/phase0/epoch_processing/slashings/*/*",
//         "eth2.0-spec-tests/tests/minimal/phase0/epoch_processing/slashings/*/*",
//     }

//     tests_for_sub_transition! {
//         final_updates,
//         wrap_in_ok(process_final_updates),
//         "eth2.0-spec-tests/tests/mainnet/phase0/epoch_processing/final_updates/*/*",
//         "eth2.0-spec-tests/tests/minimal/phase0/epoch_processing/final_updates/*/*",
//     }

//     fn wrap_in_ok<T>(
//         infallible_function: impl FnOnce(&mut T),
//     ) -> impl FnOnce(&mut T) -> Result<(), Void> {
//         |argument| Ok(infallible_function(argument))
//     }

//     fn run_case<C, E, F>(case_directory: &str, sub_transition: F)
//     where
//         C: Config,
//         E: Debug,
//         F: FnOnce(&mut BeaconState<C>) -> Result<(), E>,
//     {
//         let mut state = spec_test_utils::pre(case_directory);
//         let expected_post = spec_test_utils::post(case_directory)
//             .expect("every epoch processing test should have a post-state");

//         sub_transition(&mut state).expect("every epoch processing test should succeed");

//         assert_eq!(state, expected_post);
//     }
// }
