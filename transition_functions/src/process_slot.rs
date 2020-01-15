use crate::*;
// use blocks::block_processing::*;
use core::consts::ExpConst;
// use core::*;
use epochs::process_epoch::process_epoch;
// use ethereum_types::H256 as Hash256;
use helper_functions::{
    crypto::{
        hash_tree_root,
        signed_root,
    },
};
use types::{
    beacon_state::{
        BeaconState,
    },
    config::{
        Config, 
        // MainnetConfig
    },
    primitives::{
        Slot,
        H256,
    },
    types::{
        BeaconBlock,
    },
};
#[derive(Debug, PartialEq)]
pub enum Error {}

fn state_transition<T: Config + ExpConst>(
    state: &mut BeaconState<T>,
    block: &BeaconBlock<T>,
    validate_state_root: bool,
) -> BeaconState<T> {
    //# Process slots (including those with no blocks) since block
    process_slots(state, block.slot);
    //# Process block
    blocks::block_processing::process_block(state, block);
    //# Validate state root (`validate_state_root == True` in production)
    if validate_state_root {
        assert!(block.state_root == hash_tree_root(state));
    }
    //# Return post-state
    return state.clone();
}

fn process_slots<T: Config + ExpConst>(state: &mut BeaconState<T>, slot: Slot) {
    assert!(state.slot <= slot);
    while state.slot < slot {
        process_slot(state);
        //# Process epoch on the start slot of the next epoch
        if (state.slot + 1) % T::slots_per_epoch() == 0 {
            process_epoch(state);
        }
        state.slot += 1;
    }
}

fn process_slot<T: Config + ExpConst>(state: &mut BeaconState<T>) {
    // Cache state root
    let previous_state_root = hash_tree_root(state);

    state.state_roots[(state.slot as usize) % (T::slots_per_historical_root() as usize)] =
        previous_state_root;
    // Cache latest block header state root
    if state.latest_block_header.state_root == H256::from([0 as u8; 32]) {
        state.latest_block_header.state_root = previous_state_root;
    }
    // Cache block root
    let previous_block_root = signed_root(&state.latest_block_header);
    state.block_roots[(state.slot as usize) % (T::slots_per_historical_root() as usize)] =
        previous_block_root;
}

/*
pub fn process_slot<T: Config>(state: &mut BeaconState<T>, genesis_slot: u64) -> Result<(), Error> {
    cache_state(state)?;

    if state.slot > genesis_slot
    && (state.slot + 1) % T::slots_per_epoch() == 0
    {
        process_epoch(state);
    }

    state.slot += 1;

    Ok(())
}

fn cache_state<T: Config>(state: &mut BeaconState<T>) -> Result<(), Error> {
    let previous_state_root = state.update_tree_hash_cache().unwrap(); //?;
    let previous_slot = state.slot;

    // ! FIX THIS :( @pikaciu22x
    state.slot += 1;

    state.set_state_root(previous_slot, previous_state_root); //?;

    if state.latest_block_header.state_root == Hash256::zero() {
        state.latest_block_header.state_root = previous_state_root;
    }

    let latest_block_root = state.latest_block_header.canonical_root();
    state.set_block_root(previous_slot, latest_block_root); //?;

    state.slot -= 1;

    Ok(())
}
*/
#[cfg(test)]
mod process_slot_tests {
    use types::{beacon_state::*, config::MainnetConfig};
    // use crate::{config::*};
    use super::*;

    #[test]
    fn process_good_slot() {
        let mut bs: BeaconState<MainnetConfig> = BeaconState {
            ..BeaconState::default()
        };

        process_slots(&mut bs, 1);

        assert_eq!(bs.slot, 1);
    }
    #[test]
    fn process_good_slot_2() {
        let mut bs: BeaconState<MainnetConfig> = BeaconState {
            slot: 3,
            ..BeaconState::default()
        };

        process_slots(&mut bs, 4);
        //assert_eq!(bs.slot, 6);
    }
}
