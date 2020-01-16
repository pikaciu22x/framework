use crate::crypto::*;
use crate::math::*;
use crate::misc::*;
use crate::predicates::is_active_validator;
use ethereum_types::H256;
use ssz_types::BitList;
use std::cmp::max;
use std::collections::BTreeSet;
use std::convert::TryFrom;
use types::beacon_state::BeaconState;
use types::config::Config;
use types::consts::*;
use types::helper_functions_types::Error;
use types::primitives::*;
use types::types::{Attestation, AttestationData, IndexedAttestation};

pub fn get_current_epoch<C: Config>(state: &BeaconState<C>) -> Epoch {
    compute_epoch_at_slot::<C>(state.slot)
}

pub fn get_previous_epoch<C: Config>(state: &BeaconState<C>) -> Epoch {
    let current_epoch = get_current_epoch(state);
    if current_epoch == 0 {
        current_epoch
    } else {
        current_epoch - 1
    }
}

pub fn get_block_root<C: Config>(state: &BeaconState<C>, epoch: Epoch) -> Result<H256, Error> {
    get_block_root_at_slot::<C>(state, compute_start_slot_at_epoch::<C>(epoch))
}

pub fn get_block_root_at_slot<C: Config>(
    state: &BeaconState<C>,
    slot: Slot,
) -> Result<H256, Error> {
    if !(slot < state.slot && state.slot <= slot + SLOTS_PER_HISTORICAL_ROOT) {
        return Err(Error::SlotOutOfRange);
    }

    let index =
        usize::try_from(slot % SLOTS_PER_HISTORICAL_ROOT).expect("Expected successfull cast");

    if index >= state.block_roots.len() {
        return Err(Error::IndexOutOfRange);
    }

    Ok(state.block_roots[index])
}

pub fn get_randao_mix<C: Config>(state: &BeaconState<C>, epoch: Epoch) -> Result<H256, Error> {
    let index =
        usize::try_from(epoch % EPOCHS_PER_HISTORICAL_VECTOR).expect("Expected successfull cast");
    if index >= state.randao_mixes.len() {
        return Err(Error::IndexOutOfRange);
    }

    Ok(state.randao_mixes[index])
}

// pub fn get_active_validator_indices<C: Config>(
//     _state: &BeaconState<C>,
//     _epoch: Epoch,
// ) -> impl Iterator<Item = &ValidatorIndex> {
//     [].iter()
// }
// pub fn get_active_validator_indices<C: Config>(
//     _state: &BeaconState<C>,
//     _epoch: Epoch,
// ) -> impl Iterator<Item = &ValidatorIndex> {
//     // // let mut validators = Vec::<ValidatorIndex>::new();
//     // // let mut vals = iter::<&ValidatorIndex>();
//     // let mut vals = [].iter();
//     // for (i, v) in _state.validators.iter().enumerate() {
//     //     if is_active_validator(v, _epoch) {
//     //         // validators.push(i as ValidatorIndex);
//     //         vals.chain(&[i as ValidatorIndex]);
//     //     }
//     // }
//     // // validators.iter()
//     // vals

//     _state.validators.iter().copied().filter(|v|
//     (v, _epoch))
// }

pub fn get_active_validator_indices<C: Config>(
    state: &BeaconState<C>,
    epoch: Epoch,
) -> Vec<ValidatorIndex> {
    let mut validators = Vec::<ValidatorIndex>::new();
    for (i, v) in state.validators.iter().enumerate() {
        if is_active_validator(v, epoch) {
            validators.push(i as ValidatorIndex);
        }
    }
    validators
}

pub fn get_validator_churn_limit<C: Config>(state: &BeaconState<C>) -> Result<u64, Error> {
    let active_validator_indices = get_active_validator_indices(state, get_current_epoch(state));
    let active_validator_count = active_validator_indices.len() as u64;
    Ok(max(
        MIN_PER_EPOCH_CHURN_LIMIT,
        active_validator_count / CHURN_LIMIT_QUOTIENT,
    ))
}

pub fn get_seed<C: Config>(
    state: &BeaconState<C>,
    epoch: Epoch,
    domain_type: DomainType,
) -> Result<H256, Error> {
    let domain_bytes = int_to_bytes(domain_type.into(), 8);
    if domain_bytes.is_err() {
        return Err(domain_bytes.err().expect("Should be error"));
    }
    let domain_b = domain_bytes.expect("Expected valid conversion");

    let epoch_bytes = int_to_bytes(epoch, 8);
    if epoch_bytes.is_err() {
        return Err(epoch_bytes.err().expect("Should be error"));
    }
    let epoch_b = epoch_bytes.expect("Expected valid conversion");

    let mix = get_randao_mix(
        state,
        epoch + EPOCHS_PER_HISTORICAL_VECTOR - MIN_SEED_LOOKAHEAD - 1,
    );
    if mix.is_err() {
        return Err(mix.err().expect("Should be error"));
    }

    let mut seed: [u8; 48] = [0; 48];
    seed[0..8].copy_from_slice(&domain_b[..]);
    seed[8..16].copy_from_slice(&epoch_b[..]);
    seed[16..48].copy_from_slice(&(mix.expect("Expected success"))[..]);

    Ok(H256::from_slice(&hash(&seed)))
}

pub fn get_committee_count_at_slot<C: Config>(
    state: &BeaconState<C>,
    slot: Slot,
) -> Result<u64, Error> {
    let epoch = compute_epoch_at_slot::<C>(slot);
    let active_count = get_active_validator_indices(state, epoch).len() as u64;
    let mut count = if MAX_COMMITTEES_PER_SLOT < active_count {
        MAX_COMMITTEES_PER_SLOT
    } else {
        active_count
    };

    count = if 1 > count { 1 } else { count };

    Ok(count)
}

pub fn get_beacon_committee<C: Config>(
    state: &BeaconState<C>,
    slot: Slot,
    index: u64,
) -> Result<Vec<ValidatorIndex>, Error> {
    let epoch = compute_epoch_at_slot::<C>(slot);
    let committees_per_slot = get_committee_count_at_slot(state, slot);
    if committees_per_slot.is_err() {
        return Err(committees_per_slot.err().expect("Should be error"));
    }

    let indices = &[];
    let seed = get_seed(state, epoch, DOMAIN_BEACON_ATTESTER);
    if seed.is_err() {
        return Err(seed.err().expect("Should be error"));
    }

    let committees = committees_per_slot.expect("Expected seed");
    let i = (slot % SLOTS_PER_EPOCH) * committees + index;
    let count = committees * SLOTS_PER_EPOCH;
    compute_committee::<C>(indices, &seed.expect("Expected seed"), i, count)
}

pub fn get_beacon_proposer_index<C: Config>(
    state: &BeaconState<C>,
) -> Result<ValidatorIndex, Error> {
    let epoch = get_current_epoch(state);
    let seed = get_seed(state, epoch, DOMAIN_BEACON_PROPOSER);
    if seed.is_err() {
        return Err(seed.err().expect("Should be error"));
    }

    let indices = get_active_validator_indices(state, epoch);
    compute_proposer_index(state, &indices, &seed.expect("Expected success"))
}

pub fn get_total_balance<C: Config>(
    state: &BeaconState<C>,
    indices: &[ValidatorIndex],
) -> Result<u64, Error> {
    let mut balance: Gwei = 0;
    for (i, v) in state.validators.iter().enumerate() {
        if indices.contains(&(i as u64)) {
            balance += v.effective_balance;
        }
    }
    if balance > 1 {
        Ok(balance)
    } else {
        Ok(1)
    }
}

pub fn get_total_active_balance<C: Config>(state: &BeaconState<C>) -> Result<u64, Error> {
    let current_epoch = get_current_epoch(state);
    get_total_balance(state, &get_active_validator_indices(state, current_epoch))
}

pub fn get_domain<C: Config>(
    state: &BeaconState<C>,
    domain_type: DomainType,
    message_epoch: Option<Epoch>,
) -> Domain {
    let epoch = if message_epoch == None {
        get_current_epoch(state)
    } else {
        message_epoch.expect("Expected a value")
    };
    let fork_version = if epoch < state.fork.epoch {
        state.fork.previous_version
    } else {
        state.fork.current_version
    };
    compute_domain(domain_type, Some(&fork_version))
}

pub fn get_indexed_attestation<C: Config>(
    state: &BeaconState<C>,
    attestation: &Attestation<C>,
) -> Result<IndexedAttestation<C>, Error> {
    let attesting_indices =
        get_attesting_indices(state, &attestation.data, &attestation.aggregation_bits)?;

    let att = IndexedAttestation {
        attesting_indices: attesting_indices.into_iter().collect::<Vec<_>>().into(),
        data: attestation.data.clone(),
        signature: attestation.signature.clone(),
    };
    Ok(att)
}

pub fn get_attesting_indices<C: Config>(
    state: &BeaconState<C>,
    attestation_data: &AttestationData,
    bitlist: &BitList<C::MaxValidatorsPerCommittee>,
) -> Result<BTreeSet<ValidatorIndex>, Error> {
    let comittee = get_beacon_committee(state, attestation_data.slot, attestation_data.index);
    if comittee.is_err() {
        return Err(comittee.err().expect("Expected success"));
    }
    let mut validators: BTreeSet<ValidatorIndex> = BTreeSet::new();
    for (i, v) in comittee
        .expect("Expected success getting committee")
        .into_iter()
        .enumerate()
    {
        if bitlist.get(i).is_ok() {
            validators.insert(v);
        }
    }
    Ok(validators)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ssz_types::{typenum, FixedVector, VariableList};
    use types::config::MinimalConfig;
    use types::types::Validator;

    #[test]
    fn test_get_current_epoch() {
        let state = BeaconState::<MinimalConfig>::default();
        assert_eq!(get_current_epoch::<MinimalConfig>(&state), 0);
    }

    #[test]
    fn test_get_previous_epoch() {
        let state = BeaconState::<MinimalConfig>::default();
        assert_eq!(get_previous_epoch::<MinimalConfig>(&state), 0);
    }

    #[test]
    fn test_get_block_root() {
        let mut state = BeaconState::<MinimalConfig>::default();
        let base: Vec<H256> = vec![H256::from([0; 32])];
        let roots: FixedVector<_, typenum::U64> = FixedVector::from(base);
        state.block_roots = roots;
        let result = get_block_root::<MinimalConfig>(&state, 0);
        assert_eq!(result.is_ok(), false);
    }

    #[test]
    fn test_get_block_root_at_slot() {
        let mut state = BeaconState::<MinimalConfig>::default();
        let base: Vec<H256> = vec![H256::from([0; 32])];
        let roots: FixedVector<_, typenum::U64> = FixedVector::from(base);
        state.block_roots = roots;
        let result = get_block_root_at_slot::<MinimalConfig>(&state, 0);
        assert_eq!(result.is_ok(), false);
    }

    #[test]
    fn test_get_randao_mix() {
        let mut state = BeaconState::<MinimalConfig>::default();
        let base: Vec<H256> = vec![H256::from([0; 32])];
        let mixes: FixedVector<_, typenum::U64> = FixedVector::from(base);
        state.randao_mixes = mixes;
        let result = get_randao_mix::<MinimalConfig>(&state, 0);
        assert_eq!(result.is_ok(), true);
    }

    #[test]
    fn test_get_validator_churn_limit() {
        let state = BeaconState::<MinimalConfig>::default();
        let result = get_validator_churn_limit::<MinimalConfig>(&state);
        assert_eq!(
            result.expect("Expected MIN_PER_EPOCH_CHURN_LIMIT"),
            MIN_PER_EPOCH_CHURN_LIMIT
        );
    }

    #[test]
    fn test_get_total_balance() {
        let mut state = BeaconState::<MinimalConfig>::default();
        state.validators =
            VariableList::new([Validator::default()].to_vec()).expect("Expected success");
        let result = get_total_balance::<MinimalConfig>(&state, &[0]);
        assert_eq!(result.is_ok(), true);
        assert_eq!(result.expect("Expected success"), 1);
    }
}
