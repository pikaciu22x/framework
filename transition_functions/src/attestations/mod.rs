use helper_functions::beacon_state_accessors::{
    get_attesting_indices, get_block_root, get_block_root_at_slot, get_current_epoch,
    get_previous_epoch, get_total_balance,
};
use ssz_types::VariableList;
use types::{
    beacon_state::BeaconState,
    config::Config,
    primitives::{Epoch, Gwei, ValidatorIndex},
    types::PendingAttestation,
};

pub trait AttestableBlock<T>
where
    T: Config,
{
    fn get_matching_source_attestations(
        &self,
        epoch: Epoch,
    ) -> VariableList<PendingAttestation<T>, T::MaxAttestationsPerEpoch>;
    fn get_matching_target_attestations(
        &self,
        epoch: Epoch,
    ) -> VariableList<PendingAttestation<T>, T::MaxAttestationsPerEpoch>;
    fn get_matching_head_attestations(
        &self,
        epoch: Epoch,
    ) -> VariableList<PendingAttestation<T>, T::MaxAttestationsPerEpoch>;
    fn get_unslashed_attesting_indices(
        &self,
        attestations: VariableList<PendingAttestation<T>, T::MaxAttestationsPerEpoch>,
    ) -> VariableList<ValidatorIndex, T::MaxAttestationsPerEpoch>;
    fn get_attesting_balance(
        &self,
        attestations: VariableList<PendingAttestation<T>, T::MaxAttestationsPerEpoch>,
    ) -> Gwei;
}

impl<T> AttestableBlock<T> for BeaconState<T>
where
    T: Config,
{
    fn get_matching_source_attestations(
        &self,
        epoch: Epoch,
    ) -> VariableList<PendingAttestation<T>, T::MaxAttestationsPerEpoch> {
        assert!(epoch == get_previous_epoch(self) || epoch == get_current_epoch(self));
        if epoch == get_current_epoch(self) {
            self.current_epoch_attestations.clone()
        } else {
            self.previous_epoch_attestations.clone()
        }
    }
    fn get_matching_target_attestations(
        &self,
        epoch: Epoch,
    ) -> VariableList<PendingAttestation<T>, T::MaxAttestationsPerEpoch> {
        let mut target_attestations: VariableList<
            PendingAttestation<T>,
            T::MaxAttestationsPerEpoch,
        > = VariableList::from(vec![]);
        for attestation in self.get_matching_source_attestations(epoch).iter() {
            if attestation.data.target.root == get_block_root(self, epoch).expect("Root error") {
                target_attestations
                    .push(attestation.clone())
                    .expect("Push error");
            }
        }
        target_attestations
    }
    fn get_matching_head_attestations(
        &self,
        epoch: Epoch,
    ) -> VariableList<PendingAttestation<T>, T::MaxAttestationsPerEpoch> {
        let mut head_attestations: VariableList<PendingAttestation<T>, T::MaxAttestationsPerEpoch> =
            VariableList::from(vec![]);

        for attestation in self.get_matching_source_attestations(epoch).iter() {
            if attestation.data.beacon_block_root
                == get_block_root_at_slot(self, attestation.data.slot).expect("Root error")
            {
                head_attestations
                    .push(attestation.clone())
                    .expect("Root error");
            }
        }
        head_attestations
    }
    fn get_unslashed_attesting_indices(
        &self,
        attestations: VariableList<PendingAttestation<T>, T::MaxAttestationsPerEpoch>,
    ) -> VariableList<ValidatorIndex, T::MaxAttestationsPerEpoch> {
        let mut output: VariableList<ValidatorIndex, T::MaxAttestationsPerEpoch> =
            VariableList::from(vec![]);
        for attestation in attestations.iter() {
            let indices =
                get_attesting_indices(self, &attestation.data, &attestation.aggregation_bits)
                    .expect("Attesting indices error");
            for index in indices {
                if !(self.validators[index as usize].slashed) {
                    output.push(index).expect("Root error");
                }
            }
        }
        output
    }
    fn get_attesting_balance(
        &self,
        attestations: VariableList<PendingAttestation<T>, T::MaxAttestationsPerEpoch>,
    ) -> Gwei {
        get_total_balance(self, &self.get_unslashed_attesting_indices(attestations))
            .expect("Unslashed indices error")
    }
}

#[cfg(test)]

mod attestations_tests {
    use crate::attestations::AttestableBlock;
    use types::{beacon_state::BeaconState, config::MainnetConfig, types::PendingAttestation};

    #[test]
    fn test_get_matching_source_attestations_1() {
        let mut bs: BeaconState<MainnetConfig> = BeaconState {
            ..BeaconState::default()
        };
        let mut pa: PendingAttestation<MainnetConfig> = PendingAttestation {
            ..PendingAttestation::default()
        };
        bs.slot = 0;
        bs.current_epoch_attestations.push(pa);
        let result = bs.get_matching_source_attestations(0);
        assert_eq!(result, bs.current_epoch_attestations);
    }

    #[test]
    fn test_get_matching_source_attestations_2() {
        let mut bs: BeaconState<MainnetConfig> = BeaconState {
            ..BeaconState::default()
        };
        let mut pa: PendingAttestation<MainnetConfig> = PendingAttestation {
            ..PendingAttestation::default()
        };
        bs.slot = 32;
        bs.current_epoch_attestations.push(pa);

        let result = bs.get_matching_source_attestations(0);
        assert_eq!(result, bs.previous_epoch_attestations);
        // assert_ne!(result, bs.previous_epoch_attestations);
    }

    // #[test]
    // fn test_get_matching_target_attestations_1() {
    //     let mut bs: BeaconState<MainnetConfig> = BeaconState {
    //         ..BeaconState::default()
    //     };
    //     let mut pa: PendingAttestation<MainnetConfig> = PendingAttestation {
    //         ..PendingAttestation::default()
    //     };
    //     bs.slot = 1;
    //     bs.current_epoch_attestations.push(pa);

    //     let result = bs.get_matching_target_attestations(0);
    //     assert_eq!(result, bs.current_epoch_attestations);
    // }
}
