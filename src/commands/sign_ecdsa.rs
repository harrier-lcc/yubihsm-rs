//! Compute an ECDSA signature of the SHA-256 hash of the given data with the given key ID
//!
//! <https://developers.yubico.com/YubiHSM2/Commands/Sign_Data_Ecdsa.html>

use super::{Command, Response};
#[cfg(feature = "mockhsm")]
use mockhsm::MockHSM;
#[cfg(feature = "sha2")]
use session::{Session, SessionError};
#[cfg(all(feature = "sha2", not(feature = "mockhsm")))]
use sha2::{Digest, Sha256};
#[cfg(all(feature = "sha2", not(feature = "mockhsm")))]
use Connector;
use {CommandType, ObjectId};

/// Compute an ECDSA signature of the SHA-256 hash of the given data with the given key ID
#[cfg(all(feature = "sha2", not(feature = "mockhsm")))]
pub fn sign_ecdsa_sha2<C: Connector>(
    session: &mut Session<C>,
    key_id: ObjectId,
    data: &[u8],
) -> Result<ECDSASignature, SessionError> {
    session.send_encrypted_command(SignDataECDSACommand {
        key_id,
        digest: Sha256::digest(data).as_slice().into(),
    })
}

/// Compute an ECDSA signature of the SHA-256 hash of the given data with the given key ID
// NOTE: this version is enabled when we compile with MockHSM support
#[cfg(feature = "mockhsm")]
pub fn sign_ecdsa_sha2(
    session: &mut Session<MockHSM>,
    key_id: ObjectId,
    data: &[u8],
) -> Result<ECDSASignature, SessionError> {
    // When using the MockHSM, pass the unhashed raw message. This is because *ring* does not (yet)
    // provide an API for signing a raw digest. See: https://github.com/briansmith/ring/issues/253
    session.send_encrypted_command(SignDataECDSACommand {
        key_id,
        digest: data.into(),
    })
}

/// Request parameters for `commands::sign_ecdsa*`
#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct SignDataECDSACommand {
    /// ID of the key to perform the signature with
    pub key_id: ObjectId,

    /// Digest of data to be signed
    pub digest: Vec<u8>,
}

impl Command for SignDataECDSACommand {
    type ResponseType = ECDSASignature;
}

/// ECDSA signatures (ASN.1 DER encoded)
#[derive(Serialize, Deserialize, Debug)]
pub struct ECDSASignature(pub Vec<u8>);

#[allow(unknown_lints, len_without_is_empty)]
impl ECDSASignature {
    /// Unwrap inner byte vector
    pub fn into_vec(self) -> Vec<u8> {
        self.into()
    }

    /// Get length of the signature
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Get slice of the inner byte vector
    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        self.as_ref()
    }
}

impl Response for ECDSASignature {
    const COMMAND_TYPE: CommandType = CommandType::SignDataECDSA;
}

impl AsRef<[u8]> for ECDSASignature {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl Into<Vec<u8>> for ECDSASignature {
    fn into(self) -> Vec<u8> {
        self.0
    }
}
