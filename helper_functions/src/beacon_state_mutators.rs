use crate::error::Error;
use crate::misc::compute_activation_exit_epoch;
use crate::beacon_state_accessors::{get_current_epoch, get_validator_churn_limit};
use std::convert::TryFrom;
use types::beacon_state::BeaconState;
use types::config::Config;
use types::primitives::{Gwei, ValidatorIndex};

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
        Err(_err) => Err(Error::ConversionToUsizeError),
        Ok(id) => {
            if id >= state.validators.len() {
                return Err(Error::IndexOutOfRange);
            }

            if state.validators[id].exit_epoch != C::far_future_epoch() {
                return Err(Error::ValidatorExitAlreadyInitiated);
            }

            let max_exit_epoch = state.validators.into_iter()
                .filter(|v| v.exit_epoch != C::far_future_epoch())
                .map(|v| v.exit_epoch)
                .fold(0, |a, b| a.max(b));

            let mut exit_queue_epoch = max_exit_epoch.max(compute_activation_exit_epoch::<C>(get_current_epoch::<C>(state)));
            let exit_queue_churn = state.validators.into_iter()
                .filter(|v| v.exit_epoch == exit_queue_epoch)
                .count();   
            
            match usize::try_from(get_validator_churn_limit(state)?) {
                Err(_err) => Err(Error::ConversionToUsizeError),
                Ok(validator_churn_limit) => {
                    if exit_queue_churn >= validator_churn_limit {
                        exit_queue_epoch += 1;
                    }
                        
                    state.validators[id].exit_epoch = exit_queue_epoch;
                    state.validators[id].withdrawable_epoch = state.validators[id].exit_epoch + C::min_validator_withdrawability_delay();

                    return Ok(());
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
}
