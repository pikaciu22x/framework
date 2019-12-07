use core::consts::ExpConst;
use helper_functions::beacon_state_accessors::{
    get_block_root, get_current_epoch, get_previous_epoch,
};
use ssz_types::VariableList;
use types::{
    beacon_state::*,
    config::Config,
    primitives::{Epoch, Gwei, ValidatorIndex},
    types::PendingAttestation,
};

fn get_matching_source_attestations<T: Config + ExpConst>(
    state: &BeaconState<T>,
    epoch: Epoch,
) -> &VariableList<PendingAttestation<T>, T::MaxAttestationsPerEpoch> {
    assert!(epoch == get_previous_epoch(&state) || epoch == get_current_epoch(&state));
    if epoch == get_current_epoch(&state) {
        &state.current_epoch_attestations
    } else {
        &state.previous_epoch_attestations
    }
}

fn get_matching_target_attestations<T: Config + ExpConst>(
    state: &BeaconState<T>,
    epoch: Epoch,
) -> VariableList<PendingAttestation<T>, T::MaxAttestationsPerEpoch> {
    let mut target_attestations = VariableList::from(vec![]);
    for a in get_matching_source_attestations(state, epoch).iter() {
        if a.data.target.root == get_block_root(&state, epoch).unwrap() {
            target_attestations.push(a.clone()).unwrap();
        }
    }
    target_attestations
}

fn get_matching_head_attestations<T: Config>(_state: BeaconState<T>, _epoch: Epoch)
/*-> Sequence[PendingAttestation] */
{
    /*return [
        a for a in get_matching_source_attestations(state, epoch)
        if a.data.beacon_block_root == get_block_root_at_slot(state, a.data.slot)
    ]*/
}

fn get_unslashed_attesting_indices<'a, T: Config + 'a>(
    _state: BeaconState<T>,
    _attestations: impl Iterator<Item = &'a PendingAttestation<T>>,
) -> impl Iterator<Item = ValidatorIndex> {
    /*let mut output = set();  //# type: Set[ValidatorIndex]
    for a in attestations:
        output = output.union(get_attesting_indices(state, a.data, a.aggregation_bits))
    return set(filter(lambda index: not state.validators[index].slashed, output))*/
    vec![].into_iter()
}

fn get_attesting_balance<'a, T: Config + 'a>(
    _state: BeaconState<T>,
    _attestations: impl Iterator<Item = &'a PendingAttestation<T>>,
) -> Gwei {
    //!return get_total_balance(state, get_unslashed_attesting_indices(state, attestations));
    unimplemented!()
}
