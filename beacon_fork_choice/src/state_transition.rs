use anyhow::{ensure, Result};
use eth2_core::ExpConst;
use helper_functions::crypto;
use thiserror::Error;
use transition_functions::blocks::block_processing;
use types::{
    beacon_state::BeaconState,
    config::Config,
    primitives::{Slot, H256},
    types::BeaconBlock,
};

#[derive(Debug, Error)]
#[error("state root in block ({in_block:?}) does not match state ({real:?})")]
struct StateRootError {
    in_block: H256,
    real: H256,
}

pub fn state_transition<C: Config + ExpConst>(
    state: &mut BeaconState<C>,
    block: &BeaconBlock<C>,
    validate_state_root: bool,
) -> Result<()> {
    process_slots(state, block.slot)?;
    block_processing::process_block(state, block);
    if validate_state_root {
        let state_root = crypto::hash_tree_root(state);
        ensure!(
            block.state_root == state_root,
            StateRootError {
                in_block: block.state_root,
                real: state_root,
            }
        );
    }
    Ok(())
}

pub fn process_slots<C: Config>(_state: &mut BeaconState<C>, _slot: Slot) -> Result<()> {
    unimplemented!()
}
