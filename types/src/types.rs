//temporary Lighthouse SSZ and hashing implementation
use bls::PublicKeyBytes;
use serde::{Deserialize, Serialize};
use ssz_derive::{Decode, Encode};
use ssz_types::{BitList, FixedVector, VariableList};
use tree_hash::TreeHash;
use tree_hash_derive::{SignedRoot, TreeHash};
use typenum::{Sum, U1};

use crate::config::*;
use crate::consts;
use crate::primitives::*;

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize, Encode, Decode, TreeHash, SignedRoot)]
pub struct Attestation<C: Config> {
    pub aggregation_bits: BitList<C::MaxValidatorsPerCommittee>,
    pub data: AttestationData,
    pub custody_bits: BitList<C::MaxValidatorsPerCommittee>,
    pub signature: Signature,
}

#[derive(
    Clone,
    PartialEq,
    Eq,
    Debug,
    Hash,
    Deserialize,
    Serialize,
    Encode,
    Decode,
    TreeHash,
    SignedRoot,
    Default,
)]
pub struct AttestationData {
    pub beacon_block_root: H256,
    pub source: Checkpoint,
    pub target: Checkpoint,
    pub crosslink: Crosslink,
}

impl AttestationData {
    pub fn is_slashable_attestation_data(&self, data: &Self) -> bool {
        // Double vote
        (self != data && self.target.epoch == data.target.epoch) ||
        // Surround vote
        (self.source.epoch < data.source.epoch && data.target.epoch < self.target.epoch)
    }
}

#[derive(
    Clone, PartialEq, Eq, Debug, Deserialize, Serialize, Encode, Decode, TreeHash, SignedRoot,
)]
pub struct AttestationDataAndCustodyBit {
    pub data: AttestationData,
    pub custody_bit: bool,
}

#[derive(
    Clone,
    Copy,
    PartialEq,
    Eq,
    Debug,
    Default,
    Deserialize,
    Serialize,
    Encode,
    Decode,
    TreeHash,
    SignedRoot,
)]
pub struct AttestationDuty {
    pub slot: Slot,
    pub shard: Shard,
    pub committee_index: usize,
    pub committee_len: usize,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize, Encode, Decode, TreeHash)]
pub struct AttesterSlashing<C: Config> {
    pub attestation_1: IndexedAttestation<C>,
    pub attestation_2: IndexedAttestation<C>,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize, Encode, Decode, TreeHash, SignedRoot)]
pub struct BeaconBlock<C: Config> {
    pub slot: Slot,
    pub parent_root: H256,
    pub state_root: H256,
    pub body: BeaconBlockBody<C>,
    #[signed_root(skip_hashing)]
    pub signature: Signature,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize, Encode, Decode, TreeHash, SignedRoot)]
pub struct BeaconBlockBody<C: Config> {
    pub randao_reveal: Signature,
    pub eth1_data: Eth1Data,
    pub graffiti: [u8; 32],
    pub proposer_slashings: VariableList<ProposerSlashing, C::MaxProposerSlashings>,
    pub attester_slashings: VariableList<AttesterSlashing<C>, C::MaxAttesterSlashings>,
    pub attestations: VariableList<Attestation<C>, C::MaxAttestations>,
    pub deposits: VariableList<Deposit, C::MaxDeposits>,
    pub voluntary_exits: VariableList<VoluntaryExit, C::MaxVoluntaryExits>,
    pub transfers: VariableList<Transfer, C::MaxTransfers>,
}

#[derive(
    Clone,
    PartialEq,
    Eq,
    Debug,
    Deserialize,
    Serialize,
    Encode,
    Decode,
    TreeHash,
    SignedRoot,
    Default,
)]
pub struct BeaconBlockHeader {
    pub slot: Slot,
    pub parent_root: H256,
    pub state_root: H256,
    pub body_root: H256,
    pub signature: Signature,
}

#[derive(
    Clone, PartialEq, Eq, Debug, Default, Hash, Deserialize, Serialize, Encode, Decode, TreeHash,
)]
pub struct Checkpoint {
    pub epoch: Epoch,
    pub root: H256,
}

#[derive(
    Clone, PartialEq, Eq, Debug, Default, Hash, Deserialize, Serialize, Encode, Decode, TreeHash,
)]
pub struct Crosslink {
    pub shard: u64,
    pub parent_root: H256,
    pub start_epoch: Epoch,
    pub end_epoch: Epoch,
    pub data_root: H256,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize, Encode, Decode, TreeHash)]
pub struct Deposit {
    pub proof: FixedVector<H256, Sum<consts::DepositContractTreeDepth, U1>>,
    pub data: DepositData,
}

#[derive(
    Clone, PartialEq, Eq, Debug, Deserialize, Serialize, Encode, Decode, TreeHash, SignedRoot,
)]
pub struct DepositData {
    pub pubkey: PublicKeyBytes,
    pub withdrawal_credentials: H256,
    pub amount: u64,
    #[signed_root(skip_hashing)]
    pub signature: Signature,
}

#[derive(Clone, PartialEq, Eq, Debug, Default, Deserialize, Serialize, Encode, Decode, TreeHash)]
pub struct Eth1Data {
    pub deposit_root: H256,
    pub deposit_count: u64,
    pub block_hash: H256,
}

#[derive(
    Clone,
    PartialEq,
    Eq,
    Debug,
    Deserialize,
    Serialize,
    Encode,
    Decode,
    TreeHash,
    SignedRoot,
    Default,
)]
pub struct Fork {
    pub previous_version: Version,
    pub current_version: Version,
    pub epoch: Epoch,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize, Encode, Decode, TreeHash)]
pub struct HistoricalBatch<C: Config> {
    pub block_roots: FixedVector<H256, C::SlotsPerHistoricalRoot>,
    pub state_roots: FixedVector<H256, C::SlotsPerHistoricalRoot>,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize, Encode, Decode, TreeHash, SignedRoot)]
pub struct IndexedAttestation<C: Config> {
    pub custody_bit_0_indices: VariableList<u64, C::MaxValidatorsPerCommittee>,
    pub custody_bit_1_indices: VariableList<u64, C::MaxValidatorsPerCommittee>,
    pub data: AttestationData,
    #[signed_root(skip_hashing)]
    pub signature: Signature,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize, Encode, Decode, TreeHash)]
pub struct PendingAttestation<C: Config> {
    pub aggregation_bits: BitList<C::MaxValidatorsPerCommittee>,
    pub data: AttestationData,
    pub inclusion_delay: u64,
    pub proposer_index: u64,
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize, Encode, Decode, TreeHash)]
pub struct ProposerSlashing {
    pub proposer_index: u64,
    pub header_1: BeaconBlockHeader,
    pub header_2: BeaconBlockHeader,
}

#[derive(
    Clone, PartialEq, Eq, Debug, Deserialize, Serialize, Encode, Decode, TreeHash, SignedRoot,
)]
pub struct Transfer {
    pub sender: u64,
    pub recipient: u64,
    pub amount: u64,
    pub fee: u64,
    pub slot: Slot,
    pub pubkey: PublicKey,
    #[signed_root(skip_hashing)]
    pub signature: Signature,
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize, Encode, Decode, TreeHash, Default)]
pub struct Validator {
    pub pubkey: PublicKey,
    pub withdrawal_credentials: H256,
    pub effective_balance: u64,
    pub slashed: bool,
    pub activation_eligibility_epoch: Epoch,
    pub activation_epoch: Epoch,
    pub exit_epoch: Epoch,
    pub withdrawable_epoch: Epoch,
}

impl Validator {
    pub fn is_slashable_validator(&self, epoch: Epoch) -> bool {
        !self.slashed && self.activation_epoch <= epoch && epoch < self.withdrawable_epoch
    }

    pub fn is_active_validator(&self, epoch: Epoch) -> bool {
        self.activation_epoch <= epoch && epoch < self.exit_epoch
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_slashable_validator() {
        let v = Validator {
            slashed: false,
            activation_epoch: 0,
            withdrawable_epoch: 1,
            ..Validator::default()
        };
        assert_eq!(v.is_slashable_validator(0), true);
    }

    #[test]
    fn is_slashable_validator_already_slashed() {
        let v = Validator {
            slashed: true,
            activation_epoch: 0,
            withdrawable_epoch: 1,
            ..Validator::default()
        };
        assert_eq!(v.is_slashable_validator(0), false);
    }

    #[test]
    fn is_slashable_validator_activation_epoch_greater_than_epoch() {
        let v = Validator {
            slashed: false,
            activation_epoch: 1,
            withdrawable_epoch: 2,
            ..Validator::default()
        };
        assert_eq!(v.is_slashable_validator(0), false);
    }

    #[test]
    fn is_slashable_validator_withdrawable_epoch_equals_epoch() {
        let v = Validator {
            slashed: false,
            activation_epoch: 0,
            withdrawable_epoch: 1,
            ..Validator::default()
        };
        assert_eq!(v.is_slashable_validator(1), false);
    }

    #[test]
    fn is_active_validator() {
        let v = Validator {
            activation_epoch: 0,
            exit_epoch: 1,
            ..Validator::default()
        };
        assert_eq!(v.is_active_validator(0), true);
    }

    #[test]
    fn is_active_validator_activation_epoch_greater_than_epoch() {
        let v = Validator {
            activation_epoch: 1,
            exit_epoch: 2,
            ..Validator::default()
        };
        assert_eq!(v.is_active_validator(0), false);
    }

    #[test]
    fn is_active_validator_exit_epoch_equals_epoch() {
        let v = Validator {
            activation_epoch: 0,
            exit_epoch: 1,
            ..Validator::default()
        };
        assert_eq!(v.is_active_validator(1), false);
    }

    #[test]
    fn test_is_slashable_attestation_data_double_vote_false() {
        let attestation_data_1 = AttestationData {
            target: Checkpoint {
                epoch: 1,
                root: H256::from([0; 32]),
            },
            ..AttestationData::default()
        };
        let attestation_data_2 = AttestationData {
            target: Checkpoint {
                epoch: 1,
                root: H256::from([0; 32]),
            },
            ..AttestationData::default()
        };
        assert_eq!(
            attestation_data_1.is_slashable_attestation_data(&attestation_data_2),
            false
        );
    }

    #[test]
    fn test_is_slashable_attestation_data_double_vote_true() {
        let attestation_data_1 = AttestationData {
            target: Checkpoint {
                epoch: 1,
                root: H256::from([0; 32]),
            },
            ..AttestationData::default()
        };
        let attestation_data_2 = AttestationData {
            target: Checkpoint {
                epoch: 1,
                root: H256::from([1; 32]),
            },
            ..AttestationData::default()
        };
        assert_eq!(
            attestation_data_1.is_slashable_attestation_data(&attestation_data_2),
            true
        );
    }

    #[test]
    fn test_is_slashable_attestation_data_surround_vote_true() {
        let attestation_data_1 = AttestationData {
            source: Checkpoint {
                epoch: 0,
                root: H256::from([0; 32]),
            },
            target: Checkpoint {
                epoch: 3,
                root: H256::from([0; 32]),
            },
            ..AttestationData::default()
        };
        let attestation_data_2 = AttestationData {
            source: Checkpoint {
                epoch: 1,
                root: H256::from([1; 32]),
            },
            target: Checkpoint {
                epoch: 2,
                root: H256::from([0; 32]),
            },
            ..AttestationData::default()
        };
        assert_eq!(
            attestation_data_1.is_slashable_attestation_data(&attestation_data_2),
            true
        );
    }
}

#[derive(
    Clone, PartialEq, Eq, Debug, Deserialize, Serialize, Encode, Decode, TreeHash, SignedRoot,
)]
pub struct VoluntaryExit {
    pub epoch: Epoch,
    pub validator_index: u64,
    #[signed_root(skip_hashing)]
    pub signature: Signature,
}
