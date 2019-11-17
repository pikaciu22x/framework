use crate::{
    crypto::hash,
    error::Error,
    math::{bytes_to_int, int_to_bytes},
};
use std::cmp::max;
use typenum::marker_traits::Unsigned;
use types::config::Config;
use types::primitives::*;

pub fn compute_shuffled_index<C: Config>(
    mut index: ValidatorIndex,
    index_count: u64,
    seed: H256,
) -> Result<ValidatorIndex, Error> {
    if index >= index_count {
        return Err(Error::IndexOutOfRange);
    }
    for current_round in 0..C::shuffle_round_count() {
        let pivot = bytes_to_int(hash_seed_current_round(&seed[..], current_round)) % index_count;
        let flip = (pivot + index_count - index) % index_count;
        let position = max(index, flip);
        let source = hash_seed_current_round_position(&seed[..], current_round, position);
        let byte = source[((position % 256) / 8) as usize];
        let bit = (byte >> (position % 8)) % 2;
        index = if bit == 0 { index } else { flip };
    }
    Ok(index)
}

fn hash_seed_current_round(seed: &[u8], current_round: u64) -> [u8; 8] {
    let mut seed = seed.to_vec();
    seed.append(&mut int_to_bytes(current_round, 1));
    let mut bytes = [0; 8];
    bytes.copy_from_slice(&hash(&seed[..])[..8]);
    bytes
}

fn hash_seed_current_round_position(seed: &[u8], current_round: u64, position: u64) -> Vec<u8> {
    let mut seed = seed.to_vec();
    seed.append(&mut int_to_bytes(current_round, 1));
    seed.append(&mut int_to_bytes(position / 256, 4));
    hash(&seed[..])
}

pub fn compute_epoch_at_slot<C: Config>(slot: Slot) -> Epoch {
    slot / C::SlotsPerEpoch::to_u64()
}

pub fn compute_start_slot_of_epoch<C: Config>(epoch: Epoch) -> Slot {
    epoch * C::SlotsPerEpoch::to_u64()
}

pub fn compute_activation_exit_epoch<C: Config>(epoch: Epoch) -> Epoch {
    epoch + 1 + C::activation_exit_delay()
}

pub fn compute_domain<C: Config>(
    domain_type: DomainType,
    fork_version: Option<&Version>,
) -> Domain {
    let version: &Version = match fork_version {
        Some(version) => version,
        None => &[0_u8; 4],
    };

    let mut bytes = [0_u8; 8];
    (&mut bytes[0..4]).copy_from_slice(&domain_type.to_le_bytes()[0..4]);
    (&mut bytes[4..8]).copy_from_slice(version);
    bytes_to_int(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use types::config::MainnetConfig;

    #[test]
    #[allow(clippy::result_unwrap_used)]
    fn test_compute_shuffled_index() {
        for i in 0..1000 {
            let shuffled_index = compute_shuffled_index::<MainnetConfig>(i, 1000, H256::random());
            assert!(shuffled_index.is_ok());
            assert!(shuffled_index.unwrap() < 1000);
        }
    }

    #[test]
    fn test_compute_shuffled_index_index_greater_or_equal_index_count() {
        assert!(compute_shuffled_index::<MainnetConfig>(1, 1, H256::random()).is_err());
    }

    #[test]
    fn test_epoch_of_slot() {
        let expected_epoch = 2;
        let calculated_epoch = compute_epoch_at_slot::<MainnetConfig>(17);
        assert_eq!(calculated_epoch, expected_epoch);
    }

    #[test]
    fn test_compute_start_slot_of_epoch() {
        assert_eq!(
            compute_start_slot_of_epoch::<MainnetConfig>(10_u64),
            <MainnetConfig as Config>::SlotsPerEpoch::to_u64() * 10_u64
        );
    }

    #[test]
    fn test_compute_activation_exit_epoch() {
        assert_eq!(compute_activation_exit_epoch::<MainnetConfig>(0), 5);
    }

    #[test]
    fn test_compute_domain() {
        let version: Version = [0_u8, 0_u8, 0_u8, 1_u8];
        let domain_type: DomainType = 2_u32;
        let expected: u64 = 0x0100_0000_0000_0002_u64;

        assert_eq!(
            compute_domain::<MainnetConfig>(domain_type, Some(&version)),
            expected
        );
    }

    #[test]
    fn test_compute_domain_default_version() {
        let domain_type: DomainType = 2_u32;
        let expected: u64 = 0x0000_0000_0000_0002_u64;

        assert_eq!(compute_domain::<MainnetConfig>(domain_type, None), expected);
    }
}
