use types::beacon_state::BeaconState;
use types::config::Config;
use types::helper_functions_types::Error;
use types::primitives::{Gwei, ValidatorIndex};
use types::types::Validator;

// ok
pub fn increase_balance(_validator: &mut Validator, _delta: Gwei) -> Result<(), Error> {
    Ok(())
}

// ok
pub fn decrease_balance(_validator: &mut Validator, _delta: Gwei) -> Result<(), Error> {
    Ok(())
}

// ok
pub fn initiate_validator_exit<C: Config>(
    _state: &mut BeaconState<C>,
    _index: ValidatorIndex,
) -> Result<(), Error> {
    Ok(())
}

// ok
pub fn slash_validator<C: Config>(
    _state: &mut BeaconState<C>,
    _slashed_index: ValidatorIndex,
    _whistleblower_index: Option<ValidatorIndex>,
) -> Result<(), Error> {
    Ok(())
}
