use crate::beacon_state_accessors::{get_current_epoch, get_validator_churn_limit};
use crate::error::Error;
use crate::misc::compute_activation_exit_epoch;
use std::convert::TryFrom;
use types::{
    beacon_state::BeaconState,
    config::Config,
    primitives::{Gwei, ValidatorIndex},
};

pub fn increase_balance<C: Config>(state: &mut BeaconState<C>, index: ValidatorIndex, delta: Gwei) {
    match usize::try_from(index) {
        Err(_err) => {}
        Ok(id) => state.balances[id] += delta,
    }
}

pub fn decrease_balance<C: Config>(state: &mut BeaconState<C>, index: ValidatorIndex, delta: Gwei) {
    match usize::try_from(index) {
        Err(_err) => {}
        Ok(id) => {
            state.balances[id] = if delta > state.balances[id] {
                0
            } else {
                state.balances[id] - delta
            }
        }
    }
}

pub fn initiate_validator_exit<C: Config>(
    state: &mut BeaconState<C>,
    index: ValidatorIndex,
) -> Result<(), Error> {
    match usize::try_from(index) {
        Err(_err) => Err(Error::ConversionToUsize),
        Ok(id) => {
            if id >= state.validators.len() {
                return Err(Error::IndexOutOfRange);
            }

            if state.validators[id].exit_epoch != C::far_future_epoch() {
                return Err(Error::ValidatorExitAlreadyInitiated);
            }

            let max_exit_epoch = state
                .validators
                .into_iter()
                .filter(|v| v.exit_epoch != C::far_future_epoch())
                .map(|v| v.exit_epoch)
                .fold(0, std::cmp::Ord::max);

            let mut exit_queue_epoch = max_exit_epoch.max(compute_activation_exit_epoch::<C>(
                get_current_epoch::<C>(state),
            ));
            let exit_queue_churn = state
                .validators
                .into_iter()
                .filter(|v| v.exit_epoch == exit_queue_epoch)
                .count();
            match usize::try_from(get_validator_churn_limit(state)?) {
                Err(_err) => Err(Error::ConversionToUsize),
                Ok(validator_churn_limit) => {
                    if exit_queue_churn >= validator_churn_limit {
                        exit_queue_epoch += 1;
                    }
                    state.validators[id].exit_epoch = exit_queue_epoch;
                    state.validators[id].withdrawable_epoch =
                        state.validators[id].exit_epoch + C::min_validator_withdrawability_delay();

                    Ok(())
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ssz_types::VariableList;
    use types::config::MainnetConfig;
    use types::types::Validator;

    #[test]
    fn test_increase_balance() {
        let mut bs: BeaconState<MainnetConfig> = BeaconState {
            balances: VariableList::from(vec![0]),
            ..BeaconState::default()
        };
        increase_balance::<MainnetConfig>(&mut bs, 0, 1);
        assert_eq!(bs.balances[0], 1);
    }

    #[test]
    fn test_decrease_balance() {
        let mut bs: BeaconState<MainnetConfig> = BeaconState {
            balances: VariableList::from(vec![5]),
            ..BeaconState::default()
        };
        decrease_balance::<MainnetConfig>(&mut bs, 0, 3);
        assert_eq!(bs.balances[0], 2);
    }

    #[test]
    fn test_decrease_balance_to_negative() {
        let mut bs: BeaconState<MainnetConfig> = BeaconState {
            balances: VariableList::from(vec![0]),
            ..BeaconState::default()
        };
        decrease_balance::<MainnetConfig>(&mut bs, 0, 1);
        assert_eq!(bs.balances[0], 0);
    }

    #[test]
    fn test_initiate_validator_exit_out_of_range() {
        let mut bs: BeaconState<MainnetConfig> = BeaconState {
            validators: VariableList::from(vec![]),
            ..BeaconState::default()
        };

        assert_eq!(
            initiate_validator_exit::<MainnetConfig>(&mut bs, 1),
            Err(Error::IndexOutOfRange)
        );
    }

    #[test]
    fn test_initiate_validator_exit_validator_exit_already_initiated() {
        let v1 = Validator {
            activation_epoch: 1,
            exit_epoch: 2,
            ..Validator::default()
        };
        let mut bs: BeaconState<MainnetConfig> = BeaconState {
            validators: VariableList::from(vec![v1]),
            ..BeaconState::default()
        };

        assert_eq!(
            initiate_validator_exit::<MainnetConfig>(&mut bs, 0),
            Err(Error::ValidatorExitAlreadyInitiated)
        );
    }

    #[test]
    fn test_initiate_validator_exit() {
        let v1 = Validator {
            activation_epoch: 1,
            exit_epoch: 2,
            ..Validator::default()
        };
        let v2 = Validator {
            activation_epoch: 0,
            exit_epoch: u64::max_value(),
            ..Validator::default()
        };
        let mut bs: BeaconState<MainnetConfig> = BeaconState {
            validators: VariableList::from(vec![v1, v2]),
            ..BeaconState::default()
        };

        assert_eq!(initiate_validator_exit::<MainnetConfig>(&mut bs, 1), Ok(()));
        assert_eq!(bs.validators[1].exit_epoch, 5_u64);
    }
}
