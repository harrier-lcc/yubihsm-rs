use responses::Response;
use securechannel::{CommandMessage, CommandType};
use serde::ser::Serialize;
use serde::de::DeserializeOwned;
use serializers::serialize;

use {Algorithm, Capabilities, Domains, ObjectId, ObjectLabel, ObjectType};
use securechannel::Challenge;
use responses::*;

/// Structured commands (i.e. requests) which are encrypted and then sent to
/// the HSM. Every command has a corresponding `ResponseType`.
///
/// See <https://developers.yubico.com/YubiHSM2/Commands>
pub(crate) trait Command: Serialize + DeserializeOwned + Sized {
    /// Response type for this command
    type ResponseType: Response;

    /// Command ID for this command
    const COMMAND_TYPE: CommandType = Self::ResponseType::COMMAND_TYPE;
}

impl<C: Command> From<C> for CommandMessage {
    fn from(command: C) -> CommandMessage {
        Self::new(C::COMMAND_TYPE, serialize(&command).unwrap()).unwrap()
    }
}

/// Request parameters for `CommandType::Blink`
///
/// <https://developers.yubico.com/YubiHSM2/Commands/Blink.html>
#[derive(Serialize, Deserialize, Debug)]
pub struct BlinkCommand {
    /// Number of seconds to blink for
    pub num_seconds: u8,
}

impl Command for BlinkCommand {
    type ResponseType = BlinkResponse;
}

/// Request parameters for `CommandType::CreateSession`
///
/// <https://developers.yubico.com/YubiHSM2/Commands/Create_Session.html>
#[derive(Serialize, Deserialize, Debug)]
pub struct CreateSessionCommand {
    /// Authentication key ID to use
    pub auth_key_id: ObjectId,

    /// Randomly generated challenge from the host
    pub host_challenge: Challenge,
}

impl Command for CreateSessionCommand {
    type ResponseType = CreateSessionResponse;
}

/// Request parameters for `CommandType::DeleteObject`
///
/// <https://developers.yubico.com/YubiHSM2/Commands/Delete_Object.html>
#[derive(Serialize, Deserialize, Debug)]
pub struct DeleteObjectCommand {
    /// Object ID to delete
    pub object_id: ObjectId,

    /// Type of object to delete
    pub object_type: ObjectType,
}

impl Command for DeleteObjectCommand {
    type ResponseType = DeleteObjectResponse;
}

/// Request parameters for `CommandType::Echo`
///
/// <https://developers.yubico.com/YubiHSM2/Commands/Echo.html>
#[derive(Serialize, Deserialize, Debug)]
pub struct EchoCommand {
    /// Message to echo
    pub message: Vec<u8>,
}

impl Command for EchoCommand {
    type ResponseType = EchoResponse;
}

/// Request parameters for `CommandType::GenAsymmetricKey`
///
/// <https://developers.yubico.com/YubiHSM2/Commands/Generate_Asymmetric_Key.html>
#[derive(Serialize, Deserialize, Debug)]
pub struct GenAsymmetricKeyCommand {
    /// ID of the key
    pub key_id: ObjectId,

    /// Label for the key (40-bytes)
    pub label: ObjectLabel,

    /// Domains in which the key will be accessible
    pub domains: Domains,

    /// Capabilities of the key
    pub capabilities: Capabilities,

    /// Key algorithm
    pub algorithm: Algorithm,
}

impl Command for GenAsymmetricKeyCommand {
    type ResponseType = GenAsymmetricKeyResponse;
}

/// Request parameters for `CommandType::GetPubKey`
///
/// <https://developers.yubico.com/YubiHSM2/Commands/Get_Pubkey.html>
#[derive(Serialize, Deserialize, Debug)]
pub struct GetPubKeyCommand {
    /// Object ID of the key to obtain the corresponding pubkey for
    pub key_id: ObjectId,
}

impl Command for GetPubKeyCommand {
    type ResponseType = GetPubKeyResponse;
}

/// Request parameters for `CommandType::GetObjectInfo`
///
/// <https://developers.yubico.com/YubiHSM2/Commands/Delete_Object.html>
#[derive(Serialize, Deserialize, Debug)]
pub struct GetObjectInfoCommand {
    /// Object ID to obtain information about
    pub object_id: ObjectId,

    /// Type of object to obtain information about
    pub object_type: ObjectType,
}

impl Command for GetObjectInfoCommand {
    type ResponseType = GetObjectInfoResponse;
}

/// Request parameters for `CommandType::ListObjects`
///
/// <https://developers.yubico.com/YubiHSM2/Commands/List_Objects.html>
#[derive(Serialize, Deserialize, Debug)]
pub struct ListObjectsCommand {}

impl Command for ListObjectsCommand {
    type ResponseType = ListObjectsResponse;
}

/// Request parameters `CommandType::SignDataEdDSA`
#[derive(Serialize, Deserialize, Debug)]
pub struct SignDataEdDSACommand {
    /// ID of the key to perform the signature with
    pub key_id: ObjectId,

    /// Data to be signed
    pub data: Vec<u8>,
}

impl Command for SignDataEdDSACommand {
    type ResponseType = SignDataEdDSAResponse;
}

/// Request parameters for `CommandType::PutAsymmetricKey`
/// 
/// <https://developers.yubico.com/YubiHSM2/Commands/Put_Asymmetric.html>
#[derive(Serialize, Deserialize, Debug)]
pub struct PutAsymmetricCommand {
    /// ID of the key
    pub key_id: ObjectId,

    /// Label of the key (40 bytes)
    pub label: ObjectLabel,

    /// Avaliable domains of the key.
    pub domains: Domains,

    /// Capabilities of the key.
    pub capabilities: Capabilities,

    /// Key algorithm
    pub algorithm: Algorithm,

    /// The actual private key.
    pub data: Vec<u8>,
}

impl Command for PutAsymmetricCommand {
    type ResponseType = PutAsymmetricResponse;
}

/// Request parameters for `CommandType::WrapData`
/// 
/// <https://developers.yubico.com/YubiHSM2/Commands/Wrap_Data.html>
#[derive(Serialize, Deserialize, Debug)]
pub struct WrapDataCommand {
    pub key_id: ObjectId,
    pub data: Vec<u8>,
}

impl Command for WrapDataCommand {
    type ResponseType = WrapDataResponse;
}

/// Request parameters for `CommandType::PutWrapKey`
/// 
/// <https://developers.yubico.com/YubiHSM2/Commands/Put_Wrap_Key.html>
#[derive(Serialize, Deserialize, Debug)]
pub struct PutWrapKeyCommand {
    pub key_id: ObjectId,
    pub label: ObjectLabel,
    pub domains: Domains,
    pub capabilities: Capabilities,
    pub algorithm: Algorithm,
    pub dc: Capabilities,
    pub wrapkey: Vec<u8>,
}

impl Command for PutWrapKeyCommand {
    type ResponseType = PutWrapKeyResponse;
}

// Multiple paramater again.. deserialize might be an issue
/// Request parameters for `CommandType::UnwrapData`
/// 
/// <https://developers.yubico.com/YubiHSM2/Commands/Unwrap_Data.html>
#[derive(Serialize, Deserialize, Debug)]
pub struct UnwrapDataCommand {
    pub key_id: ObjectId,
    pub data: Vec<u8>,
}

impl Command for UnwrapDataCommand {
    type ResponseType = UnwrapDataResponse;
}

/// Request parameters for `CommandType::SignDataECDSA`
/// 
/// <https://developers.yubico.com/YubiHSM2/Commands/Sign_Data_Ecdsa.html>
#[derive(Serialize, Deserialize, Debug)]
pub struct SignDataECDSACommand {
    pub key_id: ObjectId,
    pub data: Vec<u8>,
}

impl Command for SignDataECDSACommand {
    type ResponseType = SignDataECDSAResponse;
}

/// Request parameters for `CommandType::GenerateWrapKey`
/// 
/// <https://developers.yubico.com/YubiHSM2/Commands/Generate_Wrap_Key.html>
#[derive(Serialize, Deserialize, Debug)]
pub struct GenerateWrapKeyCommand {
    pub key_id: ObjectId,
    pub label: ObjectLabel,
    pub domains: Domains,
    pub capabilities: Capabilities,
    pub algorithm: Algorithm,
    pub dc: Capabilities,
}

impl Command for GenerateWrapKeyCommand {
    type ResponseType = GenerateWrapKeyResponse;
}