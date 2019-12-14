use mockall::*;
use ssz_types::BitList;
use types::{
    beacon_state::BeaconState,
    config::Config,
    helper_functions_types::Error,
    primitives::*,
    types::{Attestation, AttestationData, IndexedAttestation},
};

// ok
pub fn get_current_epoch<C: Config>(_state: &BeaconState<C>) -> Epoch {
    23
}

// ok
pub fn get_previous_epoch<C: Config>(_state: &BeaconState<C>) -> Epoch {
    0
}

// ok
pub fn get_block_root<C: Config>(_state: &BeaconState<C>, _epoch: Epoch) -> Result<H256, Error> {
    Err(Error::IndexOutOfRange)
}

// ok
pub fn get_block_root_at_slot<C: Config>(
    _state: &BeaconState<C>,
    _slot: Slot,
) -> Result<H256, Error> {
    Ok(H256::from([0; 32]))
}

// ok
pub fn get_randao_mix<C: Config>(_state: &BeaconState<C>, _epoch: Epoch) -> Result<H256, Error> {
    Err(Error::IndexOutOfRange)
}

// ok
pub fn get_active_validator_indices<C: Config>(
    _state: &BeaconState<C>,
    _epoch: Epoch,
) -> impl Iterator<Item = &ValidatorIndex> {
    [].iter()
}

// ok
pub fn get_validator_churn_limit<C: Config>(_state: &BeaconState<C>) -> u64 {
    1
}

// ok
pub fn get_seed<C: Config>(
    _state: &BeaconState<C>,
    _epoch: Epoch,
    _domain_type: DomainType,
) -> Result<H256, Error> {
    Ok(H256::from([0; 32]))
}

// ok
pub fn get_committee_count_at_slot<C: Config>(
    _state: &BeaconState<C>,
    _slot: Slot,
) -> Result<u64, Error> {
    Ok(1)
}

// ok
pub fn get_beacon_committee<C: Config>(
    _state: &BeaconState<C>,
    _slot: Slot,
    _index: u64,
) -> Result<impl Iterator<Item = &ValidatorIndex>, Error> {
    Ok([].iter())
}

// ok
pub fn get_beacon_proposer_index<C: Config>(
    _state: &BeaconState<C>,
) -> Result<ValidatorIndex, Error> {
    Ok(0)
}

// ok
pub fn get_total_balance<C: Config>(
    _state: &BeaconState<C>,
    _indices: &[ValidatorIndex],
) -> Result<u64, Error> {
    Ok(1)
}

// ok
pub fn get_total_active_balance<C: Config>(_state: &BeaconState<C>) -> Result<u64, Error> {
    Ok(1)
}

// ok
pub fn get_domain<C: Config>(
    _state: &BeaconState<C>,
    _domain_type: DomainType,
    _message_epoch: Option<Epoch>,
) -> Domain {
    0
}

//ok
pub fn get_indexed_attestation<C: Config>(
    _state: &BeaconState<C>,
    _attestation: &Attestation<C>,
) -> Result<IndexedAttestation<C>, Error> {
    Err(Error::IndexOutOfRange)
}

//ok
pub fn get_attesting_indices<'a, C: Config>(
    _state: &'a BeaconState<C>,
    _attestation_data: &AttestationData,
    _bitlist: &BitList<C::MaxValidatorsPerCommittee>,
) -> Result<impl Iterator<Item = &'a ValidatorIndex>, Error> {
    Ok([].iter())
}

#[automock]
pub trait BeaconStateAccessor {
    fn get_current_epoch(&self) -> Epoch;
    fn get_previous_epoch(&self) -> Epoch;
    fn get_block_root(&self, _epoch: Epoch) -> Result<H256, Error>;
    fn get_block_root_at_slot(&self, _slot: Slot) -> Result<H256, Error>;
    fn get_total_active_balance(&self) -> Result<u64, Error>;
    fn get_total_balance(&self, _indices: &[ValidatorIndex]) -> Result<u64, Error>;
    // fn get_attesting_indices(&self, _attestation_data: &AttestationData, _bitlist: &BitList) -> Result<impl Iterator<Item = &ValidatorIndex>, Error>;
}

impl<C> BeaconStateAccessor for BeaconState<C>
where
    C: Config,
{
    fn get_current_epoch(&self) -> Epoch {
        get_current_epoch(self)
    }

    fn get_previous_epoch(&self) -> Epoch {
        get_previous_epoch(self)
    }
    fn get_block_root(&self, _epoch: Epoch) -> Result<H256, Error> {
        get_block_root(self, _epoch)
    }
    fn get_block_root_at_slot(&self, _slot: Slot) -> Result<H256, Error> {
        get_block_root_at_slot(self, _slot)
    }
    fn get_total_active_balance(&self) -> Result<u64, Error> {
        get_total_active_balance(self)
    }
    fn get_total_balance(&self, _indices: &[ValidatorIndex]) -> Result<u64, Error> {
        get_total_balance(self, _indices)
    }
    // fn get_attesting_indices(&self, _attestation_data: &AttestationData, _bitlist: &BitList) -> Result<impl Iterator<Item = &ValidatorIndex>, Error> {
    //     get_attesting_indices(&self, _attestation_data, _bitlist)
    // }
}
