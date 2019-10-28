pub fn increase_balance(state: &mut BeaconState, index: ValidatorIndex, delta: Gwei) {
    match usize::try_from(index) {
        Err(_err) => {}
        Ok(id) => state.balances[id] += delta,
    }
}

pub fn decrease_balance(state: &mut BeaconState, index: ValidatorIndex, delta: Gwei) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn increase_balance() {
        let mut bs: BeaconState<MainnetConfig> = BeaconState {
            balances: VariableList::from(vec![0]),
            ..BeaconState::default()
        };
        increase_balance(&bs, 0, 1);
        assert_eq!(bs.balances[0], 1);
    }

    #[test]
    fn test_decrease_balance() {
        let mut bs: BeaconState<MainnetConfig> = BeaconState {
            balances: VariableList::from(vec![5]),
            ..BeaconState::default()
        };
        decrease_balance(&bs, 0, 3);
        assert_eq!(bs.balances[0], 2);
    }

    #[test]
    fn test_decrease_balance_to_negative() {
        let mut bs: BeaconState<MainnetConfig> = BeaconState {
            balances: VariableList::from(vec![0]),
            ..BeaconState::default()
        };
        decrease_balance(&bs, 0, 1);
        assert_eq!(bs.balances[0], 0);
    }
}