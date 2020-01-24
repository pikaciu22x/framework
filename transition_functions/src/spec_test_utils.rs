use std::{
    io::ErrorKind,
    path::{Path, PathBuf},
};

use serde::{de::DeserializeOwned, Deserialize};
use ssz::Decode;
use types::{beacon_state::BeaconState, config::Config, primitives::Slot, types::BeaconBlock};

#[derive(Deserialize)]
struct BlocksMeta {
    blocks_count: usize,
}

pub fn pre<C: Config>(case_directory: impl AsRef<Path>) -> BeaconState<C> {
    optional_ssz(resolve(case_directory).join("pre.ssz"))
        .expect("every test should have a pre-state")
}

pub fn post<C: Config>(case_directory: impl AsRef<Path>) -> Option<BeaconState<C>> {
    optional_ssz(resolve(case_directory).join("post.ssz"))
}

pub fn slots(case_directory: impl AsRef<Path>) -> Slot {
    yaml(resolve(case_directory).join("slots.yaml"))
}

pub fn blocks<C: Config>(case_directory: impl AsRef<Path>) -> impl Iterator<Item = BeaconBlock<C>> {
    let BlocksMeta { blocks_count } = yaml(resolve(&case_directory).join("meta.yaml"));
    (0..blocks_count).map(move |index| {
        let file_name = format!("blocks_{}.ssz", index);
        optional_ssz(resolve(&case_directory).join(file_name))
            .expect("block sanity tests should have the number of blocks they claim to have")
    })
}

pub fn operation<D: Decode>(
    case_directory: impl AsRef<Path>,
    operation_name: impl AsRef<Path>,
) -> D {
    let operation_path = resolve(case_directory)
        .join(operation_name)
        .with_extension("ssz");
    optional_ssz(operation_path)
        .expect("every operation test should have a file representing the operation")
}

fn resolve(case_directory_relative_to_repository_root: impl AsRef<Path>) -> PathBuf {
    // Cargo appears to set the working directory to the crate root when running tests.
    PathBuf::from("..").join(case_directory_relative_to_repository_root)
}

fn optional_ssz<D: Decode>(file_path: impl AsRef<Path>) -> Option<D> {
    let bytes = std::fs::read(file_path)
        .map(Some)
        .or_else(|error| {
            if error.kind() == ErrorKind::NotFound {
                Ok(None)
            } else {
                Err(error)
            }
        })
        .expect("could not read the file")?;
    Some(D::from_ssz_bytes(bytes.as_slice()).expect("the file should contain an SSZ encoded value"))
}

fn yaml<D: DeserializeOwned>(file_path: impl AsRef<Path>) -> D {
    let bytes = std::fs::read(file_path).expect("could not read the file");
    serde_yaml::from_slice(bytes.as_slice()).expect("the file should contain a YAML encoded value")
}
