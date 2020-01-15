use core::consts::ExpConst;

use helper_functions::{
    beacon_state_accessors::{
        get_current_epoch,
        get_domain,
        get_beacon_proposer_index,
        get_randao_mix,
        get_indexed_attestation,
        get_beacon_committee,
        get_committee_count_at_slot,
        get_previous_epoch,
    },
    beacon_state_mutators::{
        slash_validator,
        initiate_validator_exit,
        increase_balance,
    },
    crypto::{
        bls_verify, 
        hash, 
        hash_tree_root, 
        signed_root,
    },
    math::{
        xor,
    },
    misc::{
        compute_domain,
        compute_epoch_at_slot,
    },
    predicates::{
        is_active_validator, 
        is_slashable_attestation_data, 
        is_slashable_validator,
        is_valid_indexed_attestation, 
        is_valid_merkle_branch,
    },
};
use std::collections::BTreeSet;
use std::convert::TryInto;
use types::{
    beacon_state::{
        BeaconState,
        Error,
    },
    config::{
        Config, 
        // MainnetConfig
    },
    consts::{
        MAX_DEPOSITS,
        SLOTS_PER_ETH1_VOTING_PERIOD,
        EPOCHS_PER_HISTORICAL_VECTOR,
        DEPOSIT_CONTRACT_TREE_DEPTH,
    },
    types::{
        VoluntaryExit,
        BeaconBlock,
        BeaconBlockBody,
        PendingAttestation,
        Attestation,
        AttestationData,
        AttesterSlashing,
        BeaconBlockHeader,
        ProposerSlashing,
        Deposit,
        Validator,
    },
};

pub fn process_block<T: Config + ExpConst>(state: &mut BeaconState<T>, block: &BeaconBlock<T>) {
    process_block_header(state, &block);
    process_randao(state, &block.body);
    process_eth1_data(state, &block.body);
    process_operations(state, &block.body);
}

fn process_voluntary_exit<T: Config + ExpConst>(state: &mut BeaconState<T>, exit: &VoluntaryExit) {
    let validator = &state.validators[exit.validator_index as usize];
    // Verify the validator is active
    assert!(is_active_validator(&validator, get_current_epoch(state)));
    // Verify the validator has not yet exited
    assert!(validator.exit_epoch == T::far_future_epoch());
    // Exits must specify an epoch when they become valid; they are not valid before then
    assert!(get_current_epoch(state) >= exit.epoch);
    // Verify the validator has been active long enough
    assert!(
        get_current_epoch(state) >= validator.activation_epoch + T::persistent_committee_period()
    );
    // Verify signature
    let domain = get_domain(state, T::domain_voluntary_exit() as u32, Some(exit.epoch));
    assert!(bls_verify(
        &(bls::PublicKeyBytes::from_bytes(&validator.pubkey.as_bytes()).unwrap()),
        signed_root(exit).as_bytes(),
        &(exit.signature.clone()).try_into().unwrap(),
        domain
    )
    .unwrap());
    // Initiate exit
    initiate_validator_exit(state, exit.validator_index).unwrap();
}

fn process_deposit<T: Config + ExpConst>(state: &mut BeaconState<T>, deposit: &Deposit) {
    //# Verify the Merkle branch  is_valid_merkle_branch

    assert!(is_valid_merkle_branch::<T>(
        &hash_tree_root(&deposit.data),
        &deposit.proof,
        DEPOSIT_CONTRACT_TREE_DEPTH + 1,
        state.eth1_deposit_index,
        &state.eth1_data.deposit_root
    )
    .unwrap());

    //# Deposits must be processed in order
    state.eth1_deposit_index += 1;

    let pubkey = (&deposit.data.pubkey).try_into().unwrap();
    // let pubkey = bls::PublicKey::from_bytes(&deposit.data.pubkey.clone().as_bytes()).unwrap();
    let amount = deposit.data.amount;

    for validator in state.validators.iter_mut() {
        // if bls::PublicKeyBytes::from_bytes(&v.pubkey.as_bytes()).unwrap() == *pubkey {
        if validator.pubkey == pubkey {
            //# Increase balance by deposit amount
            increase_balance(validator, amount).unwrap();
            return;
        }
    }
    //# Verify the deposit signature (proof of possession) for new validators.
    //# Note: The deposit contract does not check signatures.
    //# Note: Deposits are valid across forks, thus the deposit domain is retrieved directly from `compute_domain`.
    let domain = compute_domain::<T>(T::domain_deposit() as u32, None);

    if !bls_verify(
        &pubkey.clone().try_into().unwrap(),
        signed_root(&deposit.data).as_bytes(),
        &deposit.data.signature,
        domain,
    )
    .unwrap()
    {
        return;
    }

    //# Add validator and balance entries
    // bls::PublicKey::from_bytes(&pubkey.as_bytes()).unwrap()
    state
        .validators
        .push(Validator {
            pubkey: pubkey,
            withdrawal_credentials: deposit.data.withdrawal_credentials,
            activation_eligibility_epoch: T::far_future_epoch(),
            activation_epoch: T::far_future_epoch(),
            exit_epoch: T::far_future_epoch(),
            withdrawable_epoch: T::far_future_epoch(),
            effective_balance: std::cmp::min(
                amount - (amount % T::effective_balance_increment()),
                T::max_effective_balance(),
            ),
            slashed: false,
        })
        .unwrap();
    state.balances.push(amount).unwrap();
}

fn process_block_header<T: Config + ExpConst>(state: &mut BeaconState<T>, block: &BeaconBlock<T>) {
    //# Verify that the slots match
    assert!(block.slot == state.slot);
    //# Verify that the parent matches
    assert!(block.parent_root == signed_root(&state.latest_block_header));
    //# Save current block as the new latest block
    state.latest_block_header = BeaconBlockHeader {
        slot: block.slot,
        parent_root: block.parent_root,
        //# `state_root` is zeroed and overwritten in the next `process_slot` call
        body_root: hash_tree_root(&block.body),
        //# `signature` is zeroed
        signature: block.signature.clone(), //# Placeholder
        state_root: block.state_root, //# Placeholder so the compiler doesn't scream at me as much
    };
    //# Verify proposer is not slashed
    let proposer = &state.validators[get_beacon_proposer_index(&state).unwrap() as usize];
    assert!(!proposer.slashed);
    //# Verify proposer signature
    assert!(bls_verify(
        &bls::PublicKeyBytes::from_bytes(&proposer.pubkey.as_bytes()).unwrap(),
        signed_root(block).as_bytes(),
        &block.signature.clone().try_into().unwrap(),
        get_domain(&state, T::domain_beacon_proposer() as u32, None)
    )
    .unwrap());
}

fn process_randao<T: Config + ExpConst>(state: &mut BeaconState<T>, body: &BeaconBlockBody<T>) {
    let epoch = get_current_epoch(&state);
    //# Verify RANDAO reveal
    let proposer = &state.validators[get_beacon_proposer_index(&state).unwrap() as usize];
    assert!(bls_verify(
        &(proposer.pubkey.clone()).try_into().unwrap(),
        hash_tree_root(&epoch).as_bytes(),
        &(body.randao_reveal.clone()).try_into().unwrap(),
        get_domain(&state, T::domain_randao() as u32, None)
    )
    .unwrap());
    //# Mix in RANDAO reveal
    let mix = xor(
        get_randao_mix(&state, epoch).unwrap().as_bytes(),
        &hash(&body.randao_reveal.as_bytes()),
    )
    .unwrap();
    let mut array = [0; 32];
    let mix = &mix[..array.len()]; // panics if not enough data
    array.copy_from_slice(mix);
    state.randao_mixes[(epoch % EPOCHS_PER_HISTORICAL_VECTOR) as usize] = array.try_into().unwrap();
}

fn process_proposer_slashing<T: Config + ExpConst>(
    state: &mut BeaconState<T>,
    proposer_slashing: &ProposerSlashing,
) {
    let proposer = &state.validators[proposer_slashing.proposer_index as usize];
    // Verify slots match
    assert_eq!(
        proposer_slashing.header_1.slot,
        proposer_slashing.header_2.slot
    );
    // But the headers are different
    assert_ne!(proposer_slashing.header_1, proposer_slashing.header_2);
    // Check proposer is slashable
    assert!(is_slashable_validator(&proposer, get_current_epoch(state)));
    // Signatures are valid
    let headers: [BeaconBlockHeader; 2] = [
        proposer_slashing.header_1.clone(),
        proposer_slashing.header_2.clone(),
    ];
    for header in &headers {
        let domain = get_domain(
            state,
            T::domain_beacon_proposer() as u32,
            Some(compute_epoch_at_slot::<T>(header.slot)),
        );
        //# Sekanti eilutė tai ******* amazing. signed_root helperiuose užkomentuota
        assert!(bls_verify(
            &(proposer.pubkey.clone()).try_into().unwrap(),
            signed_root(header).as_bytes(),
            &(header.signature.clone()).try_into().unwrap(),
            domain
        )
        .unwrap());
    }

    slash_validator(state, proposer_slashing.proposer_index, None).unwrap();
}

fn process_attester_slashing<T: Config + ExpConst>(
    state: &mut BeaconState<T>,
    attester_slashing: &AttesterSlashing<T>,
) {
    let attestation_1 = &attester_slashing.attestation_1;
    let attestation_2 = &attester_slashing.attestation_2;
    assert!(is_slashable_attestation_data(
        &attestation_1.data,
        &attestation_2.data
    ));
    assert!(is_valid_indexed_attestation(state, &attestation_1).is_ok());
    assert!(is_valid_indexed_attestation(state, &attestation_2).is_ok());

    let mut slashed_any = false;

    // Turns attesting_indices into a binary tree set. It's a set and it's ordered :)
    let attesting_indices_1 = attestation_1
        .custody_bit_0_indices
        .iter()
        .chain(&attestation_1.custody_bit_1_indices)
        .cloned()
        .collect::<BTreeSet<_>>();
    let attesting_indices_2 = attestation_2
        .custody_bit_0_indices
        .iter()
        .chain(&attestation_2.custody_bit_1_indices)
        .cloned()
        .collect::<BTreeSet<_>>();

    // let mut slashable_indices = Vec::new();

    for index in &attesting_indices_1 & &attesting_indices_2 {
        let validator = &state.validators[index as usize];

        if is_slashable_validator(&validator, get_current_epoch(state)) {
            slash_validator(state, index, None).unwrap();
            slashed_any = true;
        }
    }
    assert!(slashed_any);
}

fn get_attestation_data_index<T: Config + ExpConst>(
    state: &BeaconState<T>,
    attestation_data: &AttestationData,
) -> Result<u64, Error> {
    for (index, x) in state.current_epoch_attestations.iter().enumerate() {
        if x.data == *attestation_data {
            return Ok(index as u64);
        }
    }
    Err(Error::SlotOutOfBounds)
}

fn process_attestation<T: Config + ExpConst>(
    state: &mut BeaconState<T>,
    attestation: &Attestation<T>,
) {
    let data = &attestation.data;
    let index = get_attestation_data_index(state, data).unwrap();
    let attestation_slot = data.slot;
    assert!(index < get_committee_count_at_slot(state, attestation_slot).unwrap()); //# Nėra index ir slot. ¯\_(ツ)_/¯
    assert!(
        data.target.epoch == get_previous_epoch(state)
            || data.target.epoch == get_current_epoch(state)
    );
    assert!(
        attestation_slot + T::min_attestation_inclusion_delay() <= state.slot
            && state.slot <= attestation_slot + T::slots_per_epoch()
    );

    let committee = get_beacon_committee(state, attestation_slot, index).unwrap();
    assert_eq!(
        attestation.aggregation_bits.len(),
        attestation.custody_bits.len()
    );
    assert_eq!(attestation.custody_bits.len(), committee.count()); // Count suranda ilgį, bet nebelieka iteratoriaus. Might wanna look into that

    let pending_attestation = PendingAttestation {
        data: attestation.data.clone(),
        aggregation_bits: attestation.aggregation_bits.clone(),
        inclusion_delay: (state.slot - attestation_slot) as u64,
        proposer_index: get_beacon_proposer_index(state).unwrap(),
    };

    if data.target.epoch == get_current_epoch(state) {
        assert_eq!(data.source, state.current_justified_checkpoint);
        state
            .current_epoch_attestations
            .push(pending_attestation)
            .unwrap();
    } else {
        assert_eq!(data.source, state.previous_justified_checkpoint);
        state
            .previous_epoch_attestations
            .push(pending_attestation)
            .unwrap();
    }

    //# Check signature
    assert!(is_valid_indexed_attestation(
        &state,
        &get_indexed_attestation(&state, &attestation).unwrap()
    )
    .is_ok());
}

fn process_eth1_data<T: Config + ExpConst>(state: &mut BeaconState<T>, body: &BeaconBlockBody<T>) {
    state.eth1_data_votes.push(body.eth1_data.clone()).unwrap();
    let num_votes = state
        .eth1_data_votes
        .iter()
        .filter(|vote| *vote == &body.eth1_data)
        .count();

    if num_votes * 2 > SLOTS_PER_ETH1_VOTING_PERIOD {
        state.eth1_data = body.eth1_data.clone();
    }
}

fn process_operations<T: Config + ExpConst>(state: &mut BeaconState<T>, body: &BeaconBlockBody<T>) {
    //# Verify that outstanding deposits are processed up to the maximum number of deposits
    assert_eq!(
        body.deposits.len(),
        std::cmp::min(
            MAX_DEPOSITS,
            (state.eth1_data.deposit_count - state.eth1_deposit_index) as usize
        )
    );

    for proposer_slashing in body.proposer_slashings.iter() {
        process_proposer_slashing(state, proposer_slashing);
    }
    for attester_slashing in body.attester_slashings.iter() {
        process_attester_slashing(state, attester_slashing);
    }
    for attestation in body.attestations.iter() {
        process_attestation(state, attestation);
    }
    for deposit in body.deposits.iter() {
        process_deposit(state, deposit);
    }
    for voluntary_exit in body.voluntary_exits.iter() {
        process_voluntary_exit(state, voluntary_exit);
    }
}

#[cfg(test)]
mod scessing_tests {
    // use crate::{config::*};
    use super::*;
    use bls::{PublicKey, SecretKey};
    use ethereum_types::H256;
    use ssz_types::VariableList;
    use types::{
        beacon_state::*,
        config::{Config, MainnetConfig},
        types::{BeaconBlock, BeaconBlockHeader},
    };

    const EPOCH_MAX: u64 = u64::max_value();

    fn default_validator() -> Validator {
        Validator {
            effective_balance: 0,
            slashed: false,
            activation_eligibility_epoch: EPOCH_MAX,
            activation_epoch: 0,
            exit_epoch: EPOCH_MAX,
            withdrawable_epoch: EPOCH_MAX,
            withdrawal_credentials: H256([0; 32]),
            pubkey: PublicKey::from_secret_key(&SecretKey::random()),
        }
    }

    #[test]
    fn process_block_header_test() {
        // preparation
        let mut bs: BeaconState<MainnetConfig> = BeaconState {
            slot: 0,
            latest_block_header: BeaconBlockHeader {
                slot: 0,
                parent_root: H256::zero(),
                ..BeaconBlockHeader::default()
            },
            validators: VariableList::from(vec![default_validator()]),
            ..BeaconState::default()
        };

        let block: BeaconBlock<MainnetConfig> = BeaconBlock {
            slot: 0,
            parent_root: signed_root(&bs.latest_block_header),
            ..BeaconBlock::default()
        };

        // execution
        process_block_header(&mut bs, &block);

        // checks
        assert_eq!(bs.latest_block_header.slot, block.slot);
        assert_eq!(bs.latest_block_header.parent_root, block.parent_root);
        assert_eq!(
            bs.latest_block_header.body_root,
            hash_tree_root(&block.body)
        );
        assert_eq!(bs.latest_block_header.state_root, block.state_root);
    }

}
