use bls::{AggregatePublicKey, PublicKey, PublicKeyBytes, Signature, SignatureBytes};
use ring::digest::{digest, SHA256};
use ssz::DecodeError;
use std::convert::TryInto;
use types::primitives::Domain;

pub fn hash(input: &[u8]) -> Vec<u8> {
    digest(&SHA256, input).as_ref().to_vec()
}

pub fn bls_verify(
    pubkey: &PublicKeyBytes,
    message: &[u8],
    signature: &SignatureBytes,
    domain: Domain,
) -> Result<bool, DecodeError> {
    let public_key: PublicKey = pubkey.try_into()?;
    let signature: Signature = signature.try_into()?;

    Ok(signature.verify(message, domain, &public_key))
}

pub fn bls_aggregate_pubkeys(pubkeys: &[PublicKey]) -> AggregatePublicKey {
    let mut aggregated = AggregatePublicKey::new();
    for pubkey in pubkeys {
        aggregated.add(pubkey);
    }
    aggregated
}

#[cfg(test)]
mod tests {
    use super::*;
    use bls::SecretKey;

    #[test]
    fn test_hashing() {
        let input = b"lorem ipsum";
        let output = hash(input.as_ref());
        let output_bytes = output.as_ref();

        let expected_bytes = [
            0x5e, 0x2b, 0xf5, 0x7d, 0x3f, 0x40, 0xc4, 0xb6, 0xdf, 0x69, 0xda, 0xf1, 0x93, 0x6c,
            0xb7, 0x66, 0xf8, 0x32, 0x37, 0x4b, 0x4f, 0xc0, 0x25, 0x9a, 0x7c, 0xbf, 0xf0, 0x6e,
            0x2f, 0x70, 0xf2, 0x69,
        ];

        assert_eq!(expected_bytes, output_bytes);
    }

    #[test]
    fn test_bls_verify() {
        // Load some keys from a serialized secret key.
        let secret_key = SecretKey::from_bytes(&[
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x3e, 0x6a, 0x4c, 0x7d, 0xae, 0x8f, 0x35, 0x63, 0xfa, 0xbb, 0x9b, 0x57,
            0xd0, 0x4b, 0x4b, 0x21, 0xd3, 0xf2, 0xb9, 0xf4, 0x54, 0x4a, 0xdc, 0x7b, 0xed, 0xc6,
            0xcb, 0xb3, 0x6f, 0x03, 0x6b, 0x10,
        ])
        .expect("Byte conversion to secret key failed");
        let public_key = PublicKey::from_secret_key(&secret_key);

        let msg_string = String::from("test123");
        let message = msg_string.as_bytes();
        let domain: Domain = 2;
        let signature = Signature::new(message, domain, &secret_key);

        let pk_bytes = PublicKeyBytes::from_bytes(public_key.as_bytes().as_slice())
            .expect("Public key conversion to bytes failed");
        let sg_bytes = SignatureBytes::from_bytes(signature.as_bytes().as_slice())
            .expect("Signature key conversion to bytes failed");

        assert_eq!(bls_verify(&pk_bytes, message, &sg_bytes, domain), Ok(true));
    }
}
