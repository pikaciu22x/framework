// Lints are currently suppressed to prevent merge conflicts in case our contributors fix their code
// on their own. These attributes should be removed in the future.
#![allow(warnings)]
#![allow(clippy::all)]

pub mod attestations;
pub mod blocks;
pub mod epochs;
pub mod process_slot;
pub mod rewards_and_penalties;
mod state_builder;
