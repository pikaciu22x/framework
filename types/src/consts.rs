pub use crate::primitives::Gwei;

pub const JUSTIFICATION_BITS_LENGTH: usize = 4;
pub const SECONDS_PER_DAY: u64 = 86400;
pub const DEPOSIT_CONTRACT_TREE_DEPTH: u64 = 32;
pub const SLOTS_PER_EPOCH: u64 = 32; // prideta
pub const MAX_SEED_LOOKAHEAD: u64 = 4; // prideta
pub const FAR_FUTURE_EPOCH: u64 = u64::max_value(); // prideta
pub const SHUFFLE_ROUND_COUNT: u64 = 90; // prideta
pub type DepositContractTreeDepth = typenum::U32;
pub type JustificationBitsLength = typenum::U4;
pub const SLOTS_PER_ETH1_VOTING_PERIOD: usize = 1024;
pub const MAX_DEPOSITS: usize = 16;

pub const SLOTS_PER_HISTORICAL_ROOT: u64 = 8192;
pub const EPOCHS_PER_HISTORICAL_VECTOR: u64 = 0x0001_0000;
pub const MIN_PER_EPOCH_CHURN_LIMIT: u64 = 0x0004;
pub const CHURN_LIMIT_QUOTIENT: u64 = 0x0001_0000;
pub const MIN_SEED_LOOKAHEAD: u64 = 1;
pub const MAX_COMMITTEES_PER_SLOT: u64 = 64;

pub const DOMAIN_BEACON_ATTESTER: u32 = 1;
pub const DOMAIN_BEACON_PROPOSER: u32 = 0;
