//! Based on the naive LMD-GHOST fork choice rule implementation in the specification:
//! <https://github.com/ethereum/eth2.0-specs/blob/40cb72ec112903a28cbfc9e310e14844680476e5/specs/core/0_fork-choice.md>
//!
//! `assert`s from Python are represented by statements that either delay the processing of the
//! offending object or return `Err`. All other operations that can raise exceptions in Python
//! (like indexing into `dict`s) are represented by statements that panic on failure.

use core::{cmp::Ordering, mem};
use std::collections::{
    hash_map::{Entry, HashMap},
    BTreeMap,
};

use anyhow::{ensure, Result};
use error_utils::DebugAsError;
use eth2_core::ExpConst;
use helper_functions::{beacon_state_accessors, crypto, misc, predicates};
use log::info;
use maplit::hashmap;
use thiserror::Error;
use types::{
    config::Config,
    primitives::{Epoch, Gwei, Slot, ValidatorIndex, H256},
    types::{Attestation, BeaconBlock, Checkpoint},
    BeaconState,
};

use crate::{genesis, state_transition};

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Error)]
enum Error<C: Config> {
    #[error("slot {new_slot} is not later than {old_slot}")]
    SlotNotLater { old_slot: Slot, new_slot: Slot },
    #[error("block is not a descendant of finalized block (block: {block:?}, finalized_block: {finalized_block:?})")]
    NotDescendantOfFinalized {
        block: BeaconBlock<C>,
        finalized_block: BeaconBlock<C>,
    },
}

/// <https://github.com/ethereum/eth2.0-specs/blob/40cb72ec112903a28cbfc9e310e14844680476e5/specs/core/0_fork-choice.md#latestmessage>
type LatestMessage = Checkpoint;

#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
enum DelayedObject<C: Config> {
    BeaconBlock(BeaconBlock<C>),
    Attestation(Attestation<C>),
}

/// <https://github.com/ethereum/eth2.0-specs/blob/40cb72ec112903a28cbfc9e310e14844680476e5/specs/core/0_fork-choice.md#store>
pub struct Store<C: Config> {
    slot: Slot,
    justified_checkpoint: Checkpoint,
    finalized_checkpoint: Checkpoint,
    // `blocks` and `block_states` could be combined into a single map.
    // We've left them separate to match the specification more closely.
    blocks: HashMap<H256, BeaconBlock<C>>,
    block_states: HashMap<H256, BeaconState<C>>,
    checkpoint_states: HashMap<Checkpoint, BeaconState<C>>,
    latest_messages: HashMap<ValidatorIndex, LatestMessage>,

    // Extra fields used for delaying and retrying objects.
    delayed_until_block: HashMap<H256, Vec<DelayedObject<C>>>,
    delayed_until_slot: BTreeMap<Slot, Vec<DelayedObject<C>>>,
}

impl<C: Config + ExpConst> Store<C> {
    /// <https://github.com/ethereum/eth2.0-specs/blob/40cb72ec112903a28cbfc9e310e14844680476e5/specs/core/0_fork-choice.md#get_genesis_store>
    pub fn new(genesis_state: BeaconState<C>) -> Self {
        let genesis_block = genesis::block(&genesis_state);

        let epoch = C::genesis_epoch();
        let root = crypto::signed_root(&genesis_block);
        let checkpoint = Checkpoint { epoch, root };

        Self {
            slot: genesis_state.slot,
            justified_checkpoint: checkpoint,
            finalized_checkpoint: checkpoint,
            blocks: hashmap! {root => genesis_block},
            block_states: hashmap! {root => genesis_state.clone()},
            checkpoint_states: hashmap! {checkpoint => genesis_state},
            latest_messages: hashmap! {},

            delayed_until_slot: BTreeMap::new(),
            delayed_until_block: HashMap::new(),
        }
    }

    /// <https://github.com/ethereum/eth2.0-specs/blob/40cb72ec112903a28cbfc9e310e14844680476e5/specs/core/0_fork-choice.md#get_head>
    ///
    /// Unlike the `get_head` function in the specification, this returns the [`BeaconState`]
    /// produced after processing the current head block.
    pub fn head_state(&self) -> &BeaconState<C> {
        let mut current_root = self.justified_checkpoint.root;

        let justified_slot = Self::epoch_start_slot(self.justified_checkpoint.epoch);

        let head_root = loop {
            let mut child_with_plurality = None;

            for (&root, block) in &self.blocks {
                if block.parent_root == current_root && justified_slot < block.slot {
                    let balance = self.latest_attesting_balance(root, block);
                    child_with_plurality = Some((balance, root)).max(child_with_plurality);
                }
            }

            match child_with_plurality {
                Some((_, root)) => current_root = root,
                None => break current_root,
            }
        };

        &self.block_states[&head_root]
    }

    /// <https://github.com/ethereum/eth2.0-specs/blob/40cb72ec112903a28cbfc9e310e14844680476e5/specs/core/0_fork-choice.md#on_tick>
    ///
    /// Unlike `on_tick` in the specification, this should be called at the start of a slot instead
    /// of every second. The fork choice rule doesn't need a precise timestamp.
    pub fn on_slot(&mut self, slot: Slot) -> Result<()> {
        ensure!(
            self.slot < slot,
            Error::<C>::SlotNotLater {
                old_slot: self.slot,
                new_slot: slot
            },
        );
        self.slot = slot;
        self.retry_delayed_until_slot(slot)
    }

    /// <https://github.com/ethereum/eth2.0-specs/blob/40cb72ec112903a28cbfc9e310e14844680476e5/specs/core/0_fork-choice.md#on_block>
    pub fn on_block(&mut self, block: BeaconBlock<C>) -> Result<()> {
        // The specification uses 2 different ways to calculate what appears to be the same value:
        // - <https://github.com/ethereum/eth2.0-specs/blame/40cb72ec112903a28cbfc9e310e14844680476e5/specs/core/0_fork-choice.md#L155>
        // - <https://github.com/ethereum/eth2.0-specs/blame/40cb72ec112903a28cbfc9e310e14844680476e5/specs/core/0_fork-choice.md#L159>
        // We assume this is an oversight.
        let finalized_slot = Self::epoch_start_slot(self.finalized_checkpoint.epoch);

        // Ignore blocks older than the finalized block. This is needed to ensure that:
        // - The genesis block is accepted even though it does not represent a state transition.
        // - Blocks that are already known and are received again are accepted even if they are
        //   older than the current finalized block.
        if block.slot <= finalized_slot {
            return Ok(());
        }

        let parent_state = if let Some(state) = self.block_states.get(&block.parent_root) {
            state
        } else {
            self.delay_until_block(block.parent_root, DelayedObject::BeaconBlock(block));
            return Ok(());
        };

        if self.slot < block.slot {
            self.delay_until_slot(block.slot, DelayedObject::BeaconBlock(block));
            return Ok(());
        }

        let block_root = crypto::signed_root(&block);

        ensure!(
            self.ancestor(block_root, &block, finalized_slot) == self.finalized_checkpoint.root,
            Error::NotDescendantOfFinalized {
                block,
                finalized_block: self.blocks[&self.finalized_checkpoint.root].clone(),
            },
        );

        let mut state = parent_state.clone();
        state_transition::state_transition(&mut state, &block, true)?;
        let state = self.block_states.entry(block_root).or_insert(state);

        // Add `block` to `self.blocks` only when it's passed all checks.
        // See <https://github.com/ethereum/eth2.0-specs/issues/1288>.
        self.blocks.insert(block_root, block);

        if self.justified_checkpoint.epoch < state.current_justified_checkpoint.epoch {
            self.justified_checkpoint = state.current_justified_checkpoint;
        }

        if self.finalized_checkpoint.epoch < state.finalized_checkpoint.epoch {
            self.finalized_checkpoint = state.finalized_checkpoint;
        }

        self.retry_delayed_until_block(block_root)
    }

    /// <https://github.com/ethereum/eth2.0-specs/blob/40cb72ec112903a28cbfc9e310e14844680476e5/specs/core/0_fork-choice.md#on_attestation>
    pub fn on_attestation(&mut self, attestation: Attestation<C>) -> Result<()> {
        let target = attestation.data.target;

        let base_state = if let Some(state) = self.block_states.get(&target.root) {
            state
        } else {
            self.delay_until_block(target.root, DelayedObject::Attestation(attestation));
            return Ok(());
        };

        let target_epoch_start = Self::epoch_start_slot(target.epoch);

        if self.slot < target_epoch_start {
            self.delay_until_slot(target_epoch_start, DelayedObject::Attestation(attestation));
            return Ok(());
        }

        let entry = self.checkpoint_states.entry(target);
        let target_state = match entry {
            Entry::Occupied(occupied) => occupied.into_mut(),
            Entry::Vacant(vacant) => {
                let mut target_state = base_state.clone();
                state_transition::process_slots(&mut target_state, target_epoch_start)?;
                vacant.insert(target_state)
            }
        };

        if self.slot <= attestation.data.slot {
            self.delay_until_slot(
                attestation.data.slot,
                DelayedObject::Attestation(attestation),
            );
            return Ok(());
        }

        let new_message = LatestMessage {
            epoch: target.epoch,
            root: attestation.data.beacon_block_root,
        };

        let indexed_attestation =
            beacon_state_accessors::get_indexed_attestation(target_state, &attestation)
                .map_err(DebugAsError::new)?;

        predicates::is_valid_indexed_attestation(target_state, &indexed_attestation)
            .map_err(DebugAsError::new)?;

        let validator_indices = indexed_attestation
            .custody_bit_0_indices
            .iter()
            .chain(&indexed_attestation.custody_bit_1_indices)
            .copied();

        for index in validator_indices {
            let old_message = self.latest_messages.entry(index).or_default();
            if old_message.epoch < new_message.epoch {
                *old_message = new_message;
            }
        }

        Ok(())
    }

    pub fn block(&self, root: H256) -> Option<&BeaconBlock<C>> {
        self.blocks.get(&root)
    }

    /// <https://github.com/ethereum/eth2.0-specs/blob/40cb72ec112903a28cbfc9e310e14844680476e5/specs/core/0_fork-choice.md#get_latest_attesting_balance>
    ///
    /// The extra `block` parameter is used to avoid a redundant block lookup.
    fn latest_attesting_balance(&self, root: H256, block: &BeaconBlock<C>) -> Gwei {
        let justified_state = &self.checkpoint_states[&self.justified_checkpoint];
        let active_indices = beacon_state_accessors::get_active_validator_indices(
            justified_state,
            beacon_state_accessors::get_current_epoch(justified_state),
        );

        active_indices
            .filter_map(|index| {
                let latest_message = self.latest_messages.get(&index)?;
                Some((index, latest_message))
            })
            .filter(|(_, latest_message)| {
                let latest_message_block = &self.blocks[&latest_message.root];
                self.ancestor(latest_message.root, latest_message_block, block.slot) == root
            })
            .map(|(index, _)| justified_state.validators[*index as usize].effective_balance)
            .sum()
    }

    /// <https://github.com/ethereum/eth2.0-specs/blob/40cb72ec112903a28cbfc9e310e14844680476e5/specs/core/0_fork-choice.md#get_ancestor>
    ///
    /// The extra `block` parameter is used to avoid adding `block` to `self.blocks` before
    /// verifying it. See <https://github.com/ethereum/eth2.0-specs/issues/1288>.
    /// The parent of `block` must still be present in `self.blocks`, however.
    fn ancestor(&self, root: H256, block: &BeaconBlock<C>, slot: Slot) -> H256 {
        match block.slot.cmp(&slot) {
            Ordering::Less => H256::zero(),
            Ordering::Equal => root,
            Ordering::Greater => {
                let parent_root = block.parent_root;
                let parent_block = &self.blocks[&block.parent_root];
                self.ancestor(parent_root, parent_block, slot)
            }
        }
    }

    fn epoch_start_slot(epoch: Epoch) -> Slot {
        misc::compute_start_slot_of_epoch::<C>(epoch)
    }

    fn delay_until_block(&mut self, block_root: H256, object: DelayedObject<C>) {
        info!("object delayed until block {:?}: {:?}", block_root, object);
        self.delayed_until_block
            .entry(block_root)
            .or_default()
            .push(object)
    }

    fn delay_until_slot(&mut self, slot: Slot, object: DelayedObject<C>) {
        info!("object delayed until slot {}: {:?}", slot, object);
        self.delayed_until_slot
            .entry(slot)
            .or_default()
            .push(object)
    }

    fn retry_delayed_until_block(&mut self, block_root: H256) -> Result<()> {
        if let Some(delayed_objects) = self.delayed_until_block.remove(&block_root) {
            self.retry_delayed(delayed_objects)?;
        }
        Ok(())
    }

    fn retry_delayed_until_slot(&mut self, slot: Slot) -> Result<()> {
        let later_slots = self.delayed_until_slot.split_off(&(slot + 1));
        let fulfilled_slots = mem::replace(&mut self.delayed_until_slot, later_slots);
        for (_, objects) in fulfilled_slots {
            self.retry_delayed(objects)?;
        }
        Ok(())
    }

    // Delayed objects are retried recursively, thus a long chain of them could overflow the stack.
    // It may be that in practice only one object will be delayed for a particular reason most of
    // the time. In that case this function would effectively be tail-recursive. The same applies to
    // slots in `Store::retry_delayed_until_slot`. The `tramp` crate may be of use in that scenario.
    // Or `become`, if that ever gets implemented.
    fn retry_delayed(&mut self, objects: Vec<DelayedObject<C>>) -> Result<()> {
        for object in objects {
            info!("retrying delayed object: {:?}", object);
            match object {
                DelayedObject::BeaconBlock(block) => self.on_block(block)?,
                DelayedObject::Attestation(attestation) => self.on_attestation(attestation)?,
            }
        }
        Ok(())
    }
}

// There used to be tests here but we were forced to omit them to save time.
