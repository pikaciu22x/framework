use bls::{PublicKey, Signature};
use hex;
use ssz_types::{FixedVector, VariableList};
use std::convert::TryInto;
use std::io::prelude::*;
use types::{
    beacon_state::*,
    config::Config,
    primitives::H256,
    types::{BeaconBlockHeader, Eth1Data, Fork, Validator},
};
use yaml_rust::{yaml::Yaml, YamlLoader};

fn build_beaconState_from_yaml<T: Config>(
    from: String,
) -> Result<BeaconState<T>, yaml_rust::scanner::ScanError> {
    let mut f = std::fs::File::open(from).unwrap();
    let mut contents = String::new();
    f.read_to_string(&mut contents).unwrap();
    let docs: Vec<Yaml> = YamlLoader::load_from_str(&contents)?;

    let d = &docs[0];

    let asd = PublicKey::from_bytes(
        &hex::decode(
            d["validators"][0]["pubkey"]
                .as_str()
                .unwrap()
                .chars()
                .skip(2)
                .collect::<String>(),
        )
        .expect("Decoding failed")[..],
    );

    Ok(BeaconState::<T> {
        genesis_time: d["genesis_time"].as_i64().unwrap() as u64,
        slot: d["slot"].as_i64().unwrap() as u64,
        fork: Fork {
            previous_version: (&hex::decode(
                d["fork"]["previous_version"]
                    .as_str()
                    .unwrap()
                    .chars()
                    .skip(2)
                    .collect::<String>(),
            )
            .expect("Decoding failed")[..])
                .try_into()
                .expect("slice with incorrect length"),
            current_version: (&hex::decode(
                d["fork"]["current_version"]
                    .as_str()
                    .unwrap()
                    .chars()
                    .skip(2)
                    .collect::<String>(),
            )
            .expect("Decoding failed")[..])
                .try_into()
                .expect("slice with incorrect length"),
            epoch: d["fork"]["epoch"].as_i64().unwrap() as u64,
        },
        latest_block_header: BeaconBlockHeader {
            slot: d["latest_block_header"]["slot"].as_i64().unwrap() as u64,
            parent_root: H256::from_slice(
                &hex::decode(
                    d["latest_block_header"]["parent_root"]
                        .as_str()
                        .unwrap()
                        .chars()
                        .skip(2)
                        .collect::<String>(),
                )
                .expect("Decoding failed")[..],
            ),
            state_root: H256::from_slice(
                &hex::decode(
                    d["latest_block_header"]["state_root"]
                        .as_str()
                        .unwrap()
                        .chars()
                        .skip(2)
                        .collect::<String>(),
                )
                .expect("Decoding failed")[..],
            ),
            body_root: H256::from_slice(
                &hex::decode(
                    d["latest_block_header"]["body_root"]
                        .as_str()
                        .unwrap()
                        .chars()
                        .skip(2)
                        .collect::<String>(),
                )
                .expect("Decoding failed")[..],
            ),
            signature: Signature::empty_signature(),
        },
        block_roots: FixedVector::from(
            d["block_roots"]
                .as_vec()
                .unwrap()
                .iter()
                .map(|x| {
                    H256::from_slice(
                        &hex::decode(x.as_str().unwrap().chars().skip(2).collect::<String>())
                            .expect("Decoding failed")[..],
                    )
                })
                .collect::<Vec<_>>()
                .to_vec(),
        ),
        state_roots: FixedVector::from(
            d["state_roots"]
                .as_vec()
                .unwrap()
                .iter()
                .map(|x| {
                    H256::from_slice(
                        &hex::decode(x.as_str().unwrap().chars().skip(2).collect::<String>())
                            .expect("Decoding failed")[..],
                    )
                })
                .collect::<Vec<_>>()
                .to_vec(),
        ),
        eth1_data: Eth1Data {
            deposit_root: H256::from_slice(
                &hex::decode(
                    d["eth1_data"]["deposit_root"]
                        .as_str()
                        .unwrap()
                        .chars()
                        .skip(2)
                        .collect::<String>(),
                )
                .expect("Decoding failed")[..],
            ),
            deposit_count: d["eth1_data"]["deposit_count"].as_i64().unwrap() as u64,
            block_hash: H256::from_slice(
                &hex::decode(
                    d["eth1_data"]["block_hash"]
                        .as_str()
                        .unwrap()
                        .chars()
                        .skip(2)
                        .collect::<String>(),
                )
                .expect("Decoding failed")[..],
            ),
        },
        eth1_data_votes: VariableList::from(
            d["eth1_data_votes"]
                .as_vec()
                .unwrap()
                .iter()
                .map(|x| Eth1Data {
                    deposit_root: H256::from_slice(
                        &hex::decode(
                            x["deposit_root"]
                                .as_str()
                                .unwrap()
                                .chars()
                                .skip(2)
                                .collect::<String>(),
                        )
                        .expect("Decoding failed")[..],
                    ),
                    deposit_count: x["deposit_count"].as_i64().unwrap() as u64,
                    block_hash: H256::from_slice(
                        &hex::decode(
                            x["block_hash"]
                                .as_str()
                                .unwrap()
                                .chars()
                                .skip(2)
                                .collect::<String>(),
                        )
                        .expect("Decoding failed")[..],
                    ),
                })
                .collect::<Vec<_>>()
                .to_vec(),
        ),
        eth1_deposit_index: d["eth1_deposit_index"].as_i64().unwrap() as u64,
        validators: VariableList::from(
            d["validators"]
                .as_vec()
                .unwrap()
                .iter()
                .map(|x| Validator {
                    pubkey: PublicKey::from_bytes(
                        &hex::decode(
                            x["pubkey"]
                                .as_str()
                                .unwrap()
                                .chars()
                                .skip(2)
                                .collect::<String>(),
                        )
                        .expect("Decoding failed")[..],
                    )
                    .unwrap(),
                    withdrawal_credentials: H256::from_slice(
                        &hex::decode(
                            x["withdrawal_credentials"]
                                .as_str()
                                .unwrap()
                                .chars()
                                .skip(2)
                                .collect::<String>(),
                        )
                        .expect("Decoding failed")[..],
                    ),
                    effective_balance: x["effective_balance"].as_i64().unwrap() as u64,
                    slashed: x["slashed"].as_bool().unwrap(),
                    activation_eligibility_epoch: x["activation_eligibility_epoch"]
                        .as_i64()
                        .unwrap() as u64,
                    activation_epoch: x["activation_epoch"].as_i64().unwrap() as u64,
                    exit_epoch: x["exit_epoch"].as_i64().unwrap() as u64,
                    withdrawable_epoch: x["withdrawable_epoch"].as_i64().unwrap() as u64,
                })
                .collect::<Vec<_>>()
                .to_vec(),
        ),
        balances: VariableList::from(
            d["balances"]
                .as_vec()
                .unwrap()
                .iter()
                .map(|x| x.as_i64().unwrap() as u64)
                .collect::<Vec<_>>(),
        ),
        randao_mixes: FixedVector::from(
            d["randao_mixes"]
                .as_vec()
                .unwrap()
                .iter()
                .map(|x| {
                    H256::from_slice(
                        &hex::decode(x.as_str().unwrap().chars().skip(2).collect::<String>())
                            .expect("Decoding failed")[..],
                    )
                })
                .collect::<Vec<_>>()
                .to_vec(),
        ),
        slashings: FixedVector::from(
            d["slashings"]
                .as_vec()
                .unwrap()
                .iter()
                .map(|x| x.as_i64().unwrap() as u64)
                .collect::<Vec<_>>(),
        ),
        // justification_bits: BitVector::from_bytes(
        //     &hex::decode(
        //         (d["justification_bits"]
        //             .as_str()
        //             .unwrap()
        //             .chars()
        //             .skip(2)
        //             .collect::<String>()),
        //     )
        //     .expect("Decoding failed")[..],
        // ).unwrap(),
        ..BeaconState::<T>::default()
    })
    // println!("buf {:?}", contents);
}

#[cfg(test)]
mod tests {
    use super::build_beaconState_from_yaml;

    use types::{
        beacon_state::*,
        config::MinimalConfig,
        // types::{BeaconBlockHeader, Eth1Data, Fork, Validator},
    };

    #[test]
    fn test123() {
        let mut bs: BeaconState<MinimalConfig> = build_beaconState_from_yaml::<MinimalConfig>(
            "./src/rewards_and_penalties/tests/attestations_some_slashed/pre.yaml".to_string(),
        )
        .unwrap();
    }
}
