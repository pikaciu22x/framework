use super::{PublicKey, SecretKey};
use serde_derive::{Deserialize, Serialize};
use std::fmt;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Keypair {
    pub sk: SecretKey,
    pub pk: PublicKey,
}

impl Keypair {
    /// Instantiate a Keypair using SecretKey::random().
    pub fn random() -> Self {
        let sk = SecretKey::random();
        let pk = PublicKey::from_secret_key(&sk);
        Self { sk, pk }
    }

    pub fn identifier(&self) -> String {
        self.pk.concatenated_hex_id()
    }
}

#[allow(clippy::derive_hash_xor_eq)]
impl Hash for Keypair {
    /// Note: this is distinct from consensus serialization, it will produce a different hash.
    ///
    /// This method uses the uncompressed bytes, which are much faster to obtain than the
    /// compressed bytes required for consensus serialization.
    ///
    /// Use `ssz::SszEncode` to obtain the bytes required for consensus hashing.
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.pk.as_uncompressed_bytes().hash(state)
    }
}

impl fmt::Display for Keypair {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.pk)
    }
}
