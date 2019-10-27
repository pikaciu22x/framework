use types::primitives::{Slot, Epoch};
use types::config::{Config};
use typenum::marker_traits::Unsigned;

pub fn epoch_of_slot<C: Config>(slot: Slot) -> Epoch {
	Epoch::from(slot / C::SlotsPerEpoch::to_u64())
}

#[cfg(test)]
mod tests {
    use types::config::{MainnetConfig};
    use super::*;

    #[test]
    fn test_epoch_of_slot() {
        let expected_epoch = 2;
        let calculated_epoch = epoch_of_slot::<MainnetConfig>(Slot::from(17 as u64));
        assert_eq!(calculated_epoch, expected_epoch);
    }
}
