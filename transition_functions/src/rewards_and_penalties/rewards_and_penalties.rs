use helper_functions;
use types::{ beacon_state::*, config::{ Config }};
use core::consts::ExpConst;
use helper_functions::math::*;
use types::primitives::*;
use helper_functions::beacon_state_accessors::*;
use helper_functions::beacon_state_mutators::*;

fn get_base_reward<T: Config + ExpConst>(state: BeaconState<T>, index: ValidatorIndex) -> Gwei{
    let total_balance = get_total_active_balance(&state).unwrap();
    let effective_balance = state.validators[index as usize].effective_balance;
    (effective_balance * T::base_reward_factor() / integer_squareroot(total_balance) / T::base_rewards_per_epoch()) as Gwei
}



fn get_attestation_deltas<T: Config + ExpConst>(state: &BeaconState<T>) -> (Vec<Gwei>, Vec<Gwei>) {
    // let previous_epoch = get_previous_epoch(state);
    // let total_balance = get_total_active_balance(state);
    let mut rewards = Vec::new();
    let mut penalties = Vec::new();
    for _ in 0..(state.validators.len()) {
        rewards.push(0 as Gwei);
        penalties.push(0 as Gwei);
    }
    // let eligible_validator_indices = [
    //     ValidatorIndex(index) for index, v in enumerate(state.validators)
    //     //!if is_active_validator(v, previous_epoch) or (v.slashed and previous_epoch + 1 < v.withdrawable_epoch)
    // ];
 
    //# Micro-incentives for matching FFG source, FFG target, and head
    /*
    let matching_source_attestations = get_matching_source_attestations(state, previous_epoch);
    let matching_target_attestations = get_matching_target_attestations(state, previous_epoch);
    let matching_head_attestations = get_matching_head_attestations(state, previous_epoch);
    for attestations in (matching_source_attestations, matching_target_attestations, matching_head_attestations){
        let unslashed_attesting_indices = get_unslashed_attesting_indices(state, attestations);
        let attesting_balance = get_total_balance(state, unslashed_attesting_indices);
        for index in eligible_validator_indices{
            if index in unslashed_attesting_indices{
                rewards[index] += get_base_reward(state, index) * attesting_balance // total_balance
            }
            else{
                penalties[index] += get_base_reward(state, index);
            }
        }

    }

    //# Proposer and inclusion delay micro-rewards
    for index in get_unslashed_attesting_indices(state, matching_source_attestations){
        let attestation = min([
            a for a in matching_source_attestations
            if index in get_attesting_indices(state, a.data, a.aggregation_bits)
        ], key=lambda a: a.inclusion_delay)
        let proposer_reward = Gwei(get_base_reward(state, index) // PROPOSER_REWARD_QUOTIENT);
        rewards[attestation.proposer_index] += proposer_reward
        let max_attester_reward = get_base_reward(state, index) - proposer_reward;
        rewards[index] += Gwei(
            max_attester_reward // attestation.inclusion_delay
        );
    }
    */
    //# Inactivity penalty
    // let finality_delay = previous_epoch - state.finalized_checkpoint.epoch;
    // if finality_delay > MIN_EPOCHS_TO_INACTIVITY_PENALTY{
    //     let matching_target_attesting_indices = get_unslashed_attesting_indices(state, matching_target_attestations);
    //     for index in eligible_validator_indices{
    //         penalties[index] += Gwei(BASE_REWARDS_PER_EPOCH * get_base_reward(state, index));
    //         if index not in matching_target_attesting_indices{
    //             penalties[index] += Gwei(
    //                 state.validators[index].effective_balance * finality_delay // INACTIVITY_PENALTY_QUOTIENT
    //             );
    //         }
    //     }
    // }


    (rewards, penalties)
}

fn process_rewards_and_penalties<T: Config + ExpConst>(state: &mut BeaconState<T>) {
    if get_current_epoch(&state) == T::genesis_epoch()
    {
        return;
    }

    let (rewards, penalties) = get_attestation_deltas(state);
    for (index, validator) in state.validators.iter_mut().enumerate() {
        increase_balance(validator, rewards[index]).unwrap();
        decrease_balance(validator, penalties[index]).unwrap();
    }
}

#[test]
fn test_base_reward() {
    use types::config::MainnetConfig;

    assert_eq!(1,1);
    let mut bs: BeaconState<MainnetConfig> = BeaconState {
        ..BeaconState::default()
    };
    let mut index = 0;
    
}