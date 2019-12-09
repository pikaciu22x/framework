use eth2_core::ExpConst;
use helper_functions::crypto;
use types::{beacon_state::BeaconState, config::Config, types::BeaconBlock};

// The way the genesis block is constructed makes it possible for many parties to independently
// produce the same block. But why does the genesis block have to exist at all? Perhaps the first
// block could be proposed by a validator as well (and not necessarily in slot 0)?
pub fn block<C: Config + ExpConst>(state: &BeaconState<C>) -> BeaconBlock<C> {
    // Note that:
    // - `BeaconBlock.body.eth1_data` is not set to `state.latest_eth1_data`.
    // - `BeaconBlock.slot` is set to 0 even if `C::genesis_slot()` is not 0.
    BeaconBlock {
        state_root: crypto::hash_tree_root(state),
        ..BeaconBlock::default()
    }
}
