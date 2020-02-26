#![allow(clippy::default_trait_access)]

//temporary Lighthouse SSZ and hashing implementation
use bls::PublicKeyBytes;
use ethereum_types::H256 as Hash256;
use serde::{Deserialize, Serialize};
use ssz_new_derive::{SszDecode, SszEncode};
use ssz_types::{BitList, FixedVector, VariableList};
use tree_hash::TreeHash;
use tree_hash_derive::TreeHash;
use typenum::{Sum, U1};

use crate::config::*;
use crate::consts;
use crate::primitives::*;

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize, SszEncode, SszDecode, TreeHash)]
pub struct Attestation<C: Config> {
    pub aggregation_bits: BitList<C::MaxValidatorsPerCommittee>,
    pub data: AttestationData,
    pub signature: AggregateSignatureBytes,
}

#[derive(
    Clone,
    PartialEq,
    Eq,
    Debug,
    Hash,
    Deserialize,
    Serialize,
    SszEncode,
    SszDecode,
    TreeHash,
    Default,
)]
pub struct AttestationData {
    pub slot: Slot,
    pub index: CommitteeIndex,
    pub beacon_block_root: H256,
    pub source: Checkpoint,
    pub target: Checkpoint,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize, SszEncode, SszDecode, TreeHash)]
pub struct AttesterSlashing<C: Config> {
    pub attestation_1: IndexedAttestation<C>,
    pub attestation_2: IndexedAttestation<C>,
}

#[derive(
    Clone, PartialEq, Debug, Default, Deserialize, Serialize, SszEncode, SszDecode, TreeHash,
)]
pub struct BeaconBlock<C: Config> {
    pub slot: Slot,
    pub parent_root: H256,
    pub state_root: H256,
    pub body: BeaconBlockBody<C>,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize, SszEncode, SszDecode, TreeHash)]
pub struct BeaconBlockBody<C: Config> {
    pub randao_reveal: SignatureBytes,
    pub eth1_data: Eth1Data,
    pub graffiti: H256,
    pub proposer_slashings: VariableList<ProposerSlashing, C::MaxProposerSlashings>,
    pub attester_slashings: VariableList<AttesterSlashing<C>, C::MaxAttesterSlashings>,
    pub attestations: VariableList<Attestation<C>, C::MaxAttestations>,
    pub deposits: VariableList<Deposit, C::MaxDeposits>,
    pub voluntary_exits: VariableList<SignedVoluntaryExit, C::MaxVoluntaryExits>,
}

impl<C: Config> Default for BeaconBlockBody<C> {
    fn default() -> Self {
        Self {
            randao_reveal: SignatureBytes::empty(),
            eth1_data: Default::default(),
            graffiti: Default::default(),
            proposer_slashings: Default::default(),
            attester_slashings: Default::default(),
            attestations: Default::default(),
            deposits: Default::default(),
            voluntary_exits: Default::default(),
        }
    }
}

// impl<C: Config> Default for BeaconBlockBody<C> {
//     fn default() -> Self {
//         #[allow(clippy::default_trait_access)]
//         Self {
//             randao_reveal: Signature::empty_signature(),
//             eth1_data: Default::default(),
//             graffiti: Default::default(),
//             proposer_slashings: Default::default(),
//             attester_slashings: Default::default(),
//             attestations: Default::default(),
//             deposits: Default::default(),
//             voluntary_exits: Default::default(),
//             transfers: Default::default(),
//         }
//     }
// }

#[derive(
    Clone, PartialEq, Eq, Debug, Default, Deserialize, Serialize, SszEncode, SszDecode, TreeHash,
)]
pub struct BeaconBlockHeader {
    pub slot: Slot,
    pub parent_root: H256,
    pub state_root: H256,
    pub body_root: H256,
}

impl BeaconBlockHeader {
    pub fn canonical_root(&self) -> Hash256 {
        Hash256::from_slice(&self.tree_hash_root()[..])
    }
}

#[derive(
    Clone,
    Copy,
    PartialEq,
    Eq,
    Debug,
    Default,
    Hash,
    Deserialize,
    Serialize,
    SszEncode,
    SszDecode,
    TreeHash,
)]
pub struct Checkpoint {
    pub epoch: Epoch,
    pub root: H256,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize, SszEncode, SszDecode, TreeHash)]
pub struct Deposit {
    pub proof: FixedVector<H256, Sum<consts::DepositContractTreeDepth, U1>>,
    pub data: DepositData,
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize, SszEncode, SszDecode, TreeHash)]
pub struct DepositData {
    pub pubkey: PublicKeyBytes,
    pub withdrawal_credentials: H256,
    pub amount: u64,
    pub signature: SignatureBytes,
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize, SszEncode, SszDecode, TreeHash)]
pub struct DepositMessage {
    pub pubkey: PublicKeyBytes,
    pub withdrawal_credentials: H256,
    pub amount: Gwei,
}

#[derive(
    Clone, PartialEq, Eq, Debug, Default, Deserialize, Serialize, SszEncode, SszDecode, TreeHash,
)]
pub struct Eth1Data {
    pub deposit_root: H256,
    pub deposit_count: u64,
    pub block_hash: H256,
}

#[derive(
    Clone, PartialEq, Eq, Debug, Deserialize, Serialize, SszEncode, SszDecode, TreeHash, Default,
)]
pub struct Fork {
    pub previous_version: Version,
    pub current_version: Version,
    pub epoch: Epoch,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize, SszEncode, SszDecode, TreeHash)]
pub struct HistoricalBatch<C: Config> {
    pub block_roots: FixedVector<H256, C::SlotsPerHistoricalRoot>,
    pub state_roots: FixedVector<H256, C::SlotsPerHistoricalRoot>,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize, SszEncode, SszDecode, TreeHash)]
pub struct IndexedAttestation<C: Config> {
    pub attesting_indices: VariableList<u64, C::MaxValidatorsPerCommittee>,
    pub data: AttestationData,
    pub signature: AggregateSignatureBytes,
}

impl<C: Config> Default for IndexedAttestation<C> {
    fn default() -> Self {
        Self {
            attesting_indices: Default::default(),
            data: Default::default(),
            signature: AggregateSignatureBytes::empty(),
        }
    }
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize, SszEncode, SszDecode, TreeHash)]
pub struct PendingAttestation<C: Config> {
    pub aggregation_bits: BitList<C::MaxValidatorsPerCommittee>,
    pub data: AttestationData,
    pub inclusion_delay: u64,
    pub proposer_index: u64,
}

impl<C> Default for PendingAttestation<C>
where
    C: Config,
{
    fn default() -> Self {
        #[allow(clippy::default_trait_access)]
        Self {
            aggregation_bits: BitList::with_capacity(2048).expect("Error initializing BitList"),
            data: Default::default(),
            inclusion_delay: Default::default(),
            proposer_index: Default::default(),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize, SszEncode, SszDecode, TreeHash)]
pub struct ProposerSlashing {
    pub proposer_index: u64,
    pub signed_header_1: SignedBeaconBlockHeader,
    pub signed_header_2: SignedBeaconBlockHeader,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize, SszEncode, SszDecode, TreeHash)]
pub struct SignedBeaconBlock<C: Config> {
    pub message: BeaconBlock<C>,
    pub signature: SignatureBytes,
}

impl<C: Config> Default for SignedBeaconBlock<C> {
    fn default() -> Self {
        Self {
            message: Default::default(),
            signature: SignatureBytes::empty(),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize, SszEncode, SszDecode, TreeHash)]
pub struct SignedBeaconBlockHeader {
    pub message: BeaconBlockHeader,
    pub signature: SignatureBytes,
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize, SszEncode, SszDecode, TreeHash)]
pub struct SignedVoluntaryExit {
    pub message: VoluntaryExit,
    pub signature: SignatureBytes,
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
    SszEncode,
    SszDecode,
    TreeHash,
)]
pub struct SigningRoot {
    pub object_root: H256,
    pub domain: Domain,
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize, SszEncode, SszDecode, TreeHash)]
pub struct Validator {
    pub pubkey: PublicKeyBytes,
    pub withdrawal_credentials: H256,
    pub effective_balance: u64,
    pub slashed: bool,
    pub activation_eligibility_epoch: Epoch,
    pub activation_epoch: Epoch,
    pub exit_epoch: Epoch,
    pub withdrawable_epoch: Epoch,
}

impl Default for Validator {
    fn default() -> Self {
        Self {
            pubkey: PublicKeyBytes::empty(),
            withdrawal_credentials: Default::default(),
            effective_balance: Default::default(),
            slashed: Default::default(),
            activation_eligibility_epoch: Default::default(),
            activation_epoch: Default::default(),
            exit_epoch: Default::default(),
            withdrawable_epoch: Default::default(),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize, SszEncode, SszDecode, TreeHash)]
pub struct VoluntaryExit {
    pub epoch: Epoch,
    pub validator_index: u64,
}
