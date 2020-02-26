use crate::block_processing::process_block;
use crate::*;
use epochs::process_epoch::process_epoch;
use helper_functions::{beacon_state_accessors::*, crypto::*, misc::*};
use std::convert::TryFrom;
use typenum::Unsigned as _;
use types::{
    beacon_state::BeaconState,
    config::Config,
    primitives::{Slot, H256},
    types::SignedBeaconBlock,
};

#[derive(Debug, PartialEq)]
pub enum Error {
    Error,
}

pub fn state_transition<T: Config>(
    state: &mut BeaconState<T>,
    signed_block: &SignedBeaconBlock<T>,
    validate_result: bool,
) -> BeaconState<T> {
    let block = &signed_block.message;
    //# Process slots (including those with no blocks) since block
    process_slots(state, block.slot);
    //# Verify signature
    if validate_result {
        assert!(verify_block_signature(state, signed_block));
    }
    //# Process block
    process_block(state, block);
    //# Validate state root (`validate_state_root == True` in production)
    if validate_result {
        assert!(block.state_root == hash_tree_root(state));
    }
    //# Return post-state
    state.clone()
}

pub fn process_slots<T: Config>(state: &mut BeaconState<T>, slot: Slot) {
    // assert!(state.slot <= slot);
    while state.slot < slot {
        process_slot(state);
        //# Process epoch on the start slot of the next epoch
        if (state.slot + 1) % T::SlotsPerEpoch::U64 == 0 {
            process_epoch(state);
        }
        state.slot += 1;
    }
}

fn process_slot<T: Config>(state: &mut BeaconState<T>) {
    // Cache state root
    let previous_state_root = hash_tree_root(state);

    state.state_roots[usize::try_from(state.slot).expect("Conversion error")
        % T::SlotsPerHistoricalRoot::USIZE] = previous_state_root;
    // Cache latest block header state root
    if state.latest_block_header.state_root == H256::from_low_u64_be(0) {
        state.latest_block_header.state_root = previous_state_root;
    }
    // Cache block root
    let previous_block_root = hash_tree_root(&state.latest_block_header);
    state.block_roots[usize::try_from(state.slot).expect("Conversion error")
        % T::SlotsPerHistoricalRoot::USIZE] = previous_block_root;
}

fn verify_block_signature<C: Config>(
    state: &BeaconState<C>,
    signed_block: &SignedBeaconBlock<C>,
) -> bool {
    let index = get_beacon_proposer_index(state).expect("Failed to get beacon proposer index");
    let proposer = &state.validators[usize::try_from(index).expect("Conversion error")];
    let domain = get_domain(state, C::domain_beacon_proposer(), None);
    let signing_root = compute_signing_root(&signed_block.message, domain);
    bls_verify(
        &proposer.pubkey,
        signing_root.as_bytes(),
        &signed_block.signature,
    )
    .expect("BLS error")
}

#[cfg(test)]
mod process_slot_tests {
    use helper_functions::beacon_state_accessors::get_current_epoch;
    use ssz_types::FixedVector;
    use std::iter;
    use types::{beacon_state::*, config::MainnetConfig};

    // use crate::{config::*};
    use super::*;

    #[test]
    fn process_good_slot() {
        let mut temp: Vec<H256> = iter::repeat(H256::from_low_u64_be(0)).take(8192).collect();
        let mut bs: BeaconState<MainnetConfig> = BeaconState {
            block_roots: FixedVector::new(temp.clone()).unwrap(),
            state_roots: FixedVector::new(temp.clone()).unwrap(),
            ..BeaconState::default()
        };

        process_slots(&mut bs, 1);

        assert_eq!(bs.slot, 1);
    }
    #[test]
    fn process_good_slot_2() {
        let mut temp: Vec<H256> = iter::repeat(H256::from_low_u64_be(0)).take(8192).collect();
        let mut bs: BeaconState<MainnetConfig> = BeaconState {
            block_roots: FixedVector::new(temp.clone()).unwrap(),
            state_roots: FixedVector::new(temp.clone()).unwrap(),
            slot: 3,
            ..BeaconState::default()
        };
        process_slots(&mut bs, 4);
        assert_eq!(bs.slot, 4);
    }

    #[test]
    fn process_epoch() {
        let mut vec_1: Vec<H256> = iter::repeat(H256::from_low_u64_be(0)).take(8192).collect();
        let mut vec_2: Vec<u64> = iter::repeat(0).take(8192).collect();
        let mut vec_3: Vec<H256> = iter::repeat(H256::from_low_u64_be(0))
            .take(0x0001_0000)
            .collect();
        let mut bs: BeaconState<MainnetConfig> = BeaconState {
            block_roots: FixedVector::new(vec_1.clone()).unwrap(),
            state_roots: FixedVector::new(vec_1.clone()).unwrap(),
            slashings: FixedVector::new(vec_2.clone()).unwrap(),
            randao_mixes: FixedVector::new(vec_3.clone()).unwrap(),
            slot: 0,
            ..BeaconState::default()
        };
        process_slots(&mut bs, 32);
        assert_eq!(get_current_epoch(&bs), 1);
    }

    // #[test]
    // fn transition_state() {
    //     let mut vec_1: Vec<H256> = iter::repeat(H256::from_low_u64_be(0)).take(8192).collect();
    //     let mut vec_2: Vec<u64> = iter::repeat(0).take(8192).collect();
    //     let mut vec_3: Vec<H256> = iter::repeat(H256::from_low_u64_be(0)).take(65536).collect();
    //     let mut bs: BeaconState<MainnetConfig> = BeaconState {
    //         block_roots: FixedVector::new(vec_1.clone()).unwrap(),
    //         state_roots: FixedVector::new(vec_1.clone()).unwrap(),
    //         slashings: FixedVector::new(vec_2.clone()).unwrap(),
    //         randao_mixes: FixedVector::new(vec_3.clone()).unwrap(),
    //         slot: 0,
    //         ..BeaconState::default()
    //     };
    //     let mut bb = BeaconBlock {
    //         slot: 1,
    //         ..BeaconBlock::default()
    //     };
    //     state_transition(&mut bs, &bb, true);
    // }
}

// #[cfg(test)]
// mod spec_tests {
//     use test_generator::test_resources;
//     use types::config::MinimalConfig;

//     use super::*;

//     // We do not honor `bls_setting` in sanity tests because none of them customize it.

//     #[test_resources("eth2.0-spec-tests/tests/mainnet/phase0/sanity/slots/*/*")]
//     fn mainnet_slots(case_directory: &str) {
//         run_slots_case::<MainnetConfig>(case_directory);
//     }

//     #[test_resources("eth2.0-spec-tests/tests/minimal/phase0/sanity/slots/*/*")]
//     fn minimal_slots(case_directory: &str) {
//         run_slots_case::<MinimalConfig>(case_directory);
//     }

//     #[test_resources("eth2.0-spec-tests/tests/mainnet/phase0/sanity/blocks/*/*")]
//     fn mainnet_blocks(case_directory: &str) {
//         run_blocks_case::<MainnetConfig>(case_directory);
//     }

//     #[test_resources("eth2.0-spec-tests/tests/minimal/phase0/sanity/blocks/*/*")]
//     fn minimal_blocks(case_directory: &str) {
//         run_blocks_case::<MinimalConfig>(case_directory);
//     }

//     fn run_slots_case<C: Config>(case_directory: &str) {
//         let mut state: BeaconState<C> = spec_test_utils::pre(case_directory);
//         let last_slot = state.slot + spec_test_utils::slots(case_directory);
//         let expected_post = spec_test_utils::post(case_directory)
//             .expect("every slot sanity test should have a post-state");

//         process_slots(&mut state, last_slot);

//         assert_eq!(state, expected_post);
//     }

//     fn run_blocks_case<C: Config>(case_directory: &str) {
//         let process_blocks = || {
//             let mut state = spec_test_utils::pre(case_directory);
//             for block in spec_test_utils::blocks(case_directory) {
//                 state_transition::<C>(&mut state, &block, true);
//             }
//             state
//         };
//         match spec_test_utils::post(case_directory) {
//             Some(expected_post) => assert_eq!(process_blocks(), expected_post),
//             // The state transition code as it is now panics on error instead of returning `Result`.
//             // We have to use `std::panic::catch_unwind` to verify that state transitions fail.
//             // This may result in tests falsely succeeding.
//             None => assert!(std::panic::catch_unwind(process_blocks).is_err()),
//         }
//     }
// }
