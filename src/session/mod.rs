#[macro_use]
mod error;

use commands::*;
use connector::{Connector, HttpConfig, HttpConnector, Status as ConnectorStatus};
use responses::*;
use securechannel::{Challenge, Channel, CommandMessage, ResponseCode, ResponseMessage, StaticKeys};
use serializers::deserialize;
use super::{Algorithm, Capabilities, Domains, ObjectId, ObjectLabel, ObjectType, SessionId};
pub use self::error::{SessionError, SessionErrorKind};

/// Salt value to use with PBKDF2 when deriving static keys from a password
pub const PBKDF2_SALT: &[u8] = b"Yubico";

/// Number of PBKDF2 iterations to perform when deriving static keys
pub const PBKDF2_ITERATIONS: usize = 10_000;

/// Status message returned from healthy connectors
const CONNECTOR_STATUS_OK: &str = "OK";

/// Encrypted session with the `YubiHSM2`
///
/// Generic over connector types in case a different one needs to be swapped
/// in, which is primarily useful for substituting the `MockHSM`.
pub struct Session<C = HttpConnector>
where
    C: Connector,
{
    /// ID of this session
    id: SessionId,

    /// Encrypted channel to the HSM
    channel: Channel,

    /// Connector to send messages through
    connector: C,

    /// Optional cached static keys for reconnecting lost sessions
    // TODO: session reconnect support
    #[allow(dead_code)]
    static_keys: Option<StaticKeys>,
}

// Special casing these for HttpConnector is a bit of a hack in that default
// generics and static methods do not play well together, e.g.
//
// error[E0283]: type annotations required: cannot resolve `yubihsm::Connector`
//
// So we special case these for HttpConnector to make the API more ergonomic
impl Session<HttpConnector> {
    /// Open a new session to the HSM, authenticating with the given keypair
    pub fn create(
        connector_config: HttpConfig,
        auth_key_id: ObjectId,
        static_keys: StaticKeys,
        reconnect: bool,
    ) -> Result<Self, SessionError> {
        let connector_info = connector_config.to_string();
        let connector = HttpConnector::open(connector_config)?;
        let status = connector.status()?;

        if status.message != CONNECTOR_STATUS_OK {
            session_fail!(
                CreateFailed,
                "bad status response from {}: {}",
                connector_info,
                status.message
            );
        }

        Self::new(connector, auth_key_id, static_keys, reconnect)
    }

    /// Open a new session to the HSM, authenticating with a given password
    pub fn create_from_password(
        connector_config: HttpConfig,
        auth_key_id: ObjectId,
        password: &str,
        reconnect: bool,
    ) -> Result<Self, SessionError> {
        Self::create(
            connector_config,
            auth_key_id,
            StaticKeys::derive_from_password(password.as_bytes(), PBKDF2_SALT, PBKDF2_ITERATIONS),
            reconnect,
        )
    }
}

impl<C: Connector> Session<C> {
    /// Create a new encrypted session using the given connector, YubiHSM2 auth key ID, and
    /// static identity keys
    pub fn new(
        connector: C,
        auth_key_id: ObjectId,
        static_keys: StaticKeys,
        reconnect: bool,
    ) -> Result<Self, SessionError> {
        let host_challenge = Challenge::random();

        let command_message: CommandMessage = CreateSessionCommand {
            auth_key_id,
            host_challenge,
        }.into();

        let uuid = command_message.uuid;
        let response_body = connector.send_command(uuid, command_message.into())?;
        let response_message = ResponseMessage::parse(response_body)?;

        if response_message.is_err() {
            session_fail!(ResponseError, "HSM error: {:?}", response_message.code);
        }

        if response_message.command().unwrap() != CommandType::CreateSession {
            session_fail!(
                ProtocolError,
                "command type mismatch: expected {:?}, got {:?}",
                CommandType::CreateSession,
                response_message.command().unwrap()
            );
        }

        let session_id = response_message
            .session_id
            .ok_or_else(|| session_err!(CreateFailed, "no session ID in response"))?;

        let response: CreateSessionResponse = deserialize(response_message.data.as_ref())?;

        let channel = Channel::new(
            session_id,
            &static_keys,
            &host_challenge,
            &response.card_challenge,
        );

        // NOTE: Cryptogram implements constant-time equality comparison
        if channel.card_cryptogram() != response.card_cryptogram {
            session_fail!(AuthFailed, "card cryptogram mismatch!");
        }

        let static_keys_option = if reconnect { Some(static_keys) } else { None };

        let mut session = Self {
            id: session_id,
            channel,
            connector,
            static_keys: static_keys_option,
        };

        session.authenticate()?;
        Ok(session)
    }

    /// Request current yubihsm-connector status
    #[inline]
    pub fn connector_status(&mut self) -> Result<ConnectorStatus, SessionError> {
        self.connector.status().map_err(|e| e.into())
    }

    /// Blink the YubiHSM2's LEDs (to identify it) for the given number of seconds
    pub fn blink(&mut self, num_seconds: u8) -> Result<BlinkResponse, SessionError> {
        self.send_encrypted_command(BlinkCommand { num_seconds })
    }

    /// Delete an object of the given ID and type
    pub fn delete_object(
        &mut self,
        object_id: ObjectId,
        object_type: ObjectType,
    ) -> Result<DeleteObjectResponse, SessionError> {
        self.send_encrypted_command(DeleteObjectCommand {
            object_id,
            object_type,
        })
    }

    /// Have the card echo an input message
    pub fn echo<T>(&mut self, message: T) -> Result<EchoResponse, SessionError>
    where
        T: Into<Vec<u8>>,
    {
        self.send_encrypted_command(EchoCommand {
            message: message.into(),
        })
    }

    /// Generate a new asymmetric key within the `YubiHSM2`
    pub fn generate_asymmetric_key(
        &mut self,
        key_id: ObjectId,
        label: ObjectLabel,
        domains: Domains,
        capabilities: Capabilities,
        algorithm: Algorithm,
    ) -> Result<GenAsymmetricKeyResponse, SessionError> {
        self.send_encrypted_command(GenAsymmetricKeyCommand {
            key_id,
            label,
            domains,
            capabilities,
            algorithm,
        })
    }

    /// Get information about an object
    pub fn get_object_info(
        &mut self,
        object_id: ObjectId,
        object_type: ObjectType,
    ) -> Result<GetObjectInfoResponse, SessionError> {
        self.send_encrypted_command(GetObjectInfoCommand {
            object_id,
            object_type,
        })
    }

    /// Get the public key for an asymmetric key stored on the device
    ///
    /// See `GetPubKeyResponse` for more information about public key formats
    pub fn get_pubkey(&mut self, key_id: ObjectId) -> Result<GetPubKeyResponse, SessionError> {
        self.send_encrypted_command(GetPubKeyCommand { key_id })
    }

    /// Get the current session ID
    pub fn id(&self) -> SessionId {
        self.id
    }

    /// List objects visible from the current session
    pub fn list_objects(&mut self) -> Result<ListObjectsResponse, SessionError> {
        // TODO: support for filtering objects
        self.send_encrypted_command(ListObjectsCommand {})
    }

    /// Compute an Ed25519 signature with the given key ID
    pub fn sign_data_eddsa<T>(
        &mut self,
        key_id: ObjectId,
        data: T,
    ) -> Result<SignDataEdDSAResponse, SessionError>
    where
        T: Into<Vec<u8>>,
    {
        self.send_encrypted_command(SignDataEdDSACommand {
            key_id,
            data: data.into(),
        })
    }

    // Parser is require to break pem files to algorithm and data
    // how to implement data1 data2?
    // is RSA needed?
    // hold
    /// Put an Asymmetric Key into the device
    /// Required to put a 
    pub fn put_asymmetric<T>(
        &mut self,
        key_id: ObjectId,
        label: ObjectLabel,
        domains: Domains,
        capabilities: Capabilities,
        algorithm: Algorithm,
        data: T,
    ) -> Result<PutAsymmetricResponse, SessionError>
    where T: Into<Vec<u8>>,
    {
        self.send_encrypted_command(PutAsymmetricCommand {
            key_id,
            label,
            domains,
            capabilities,
            algorithm,
            data: data.into(),
        })
    }

    /// Import a wrapkey.
    pub fn put_wrap_key<T>(
        &mut self,
        key_id : ObjectId,
        label: ObjectLabel,
        domains: Domains,
        capabilities: Capabilities,
        algorithm: Algorithm,
        dc: Capabilities,
        wrapkey: T,
    ) -> Result<PutWrapKeyResponse, SessionError>
    where T: Into<Vec<u8>>,
    {
        self.send_encrypted_command(PutWrapKeyCommand {
            key_id,
            label,
            domains,
            capabilities,
            algorithm,
            dc,
            wrapkey: wrapkey.into(),
        })
    }

    /// Encrypt (wrap) data using a wrapkey.
    pub fn wrap_data<T>(
        &mut self,
        key_id: ObjectId,
        data: T,
    ) -> Result<WrapDataResponse, SessionError>
    where T: Into<Vec<u8>>,
    {
        self.send_encrypted_command(WrapDataCommand {
            key_id,
            data: data.into(),
        })
    }


    /// Decrypt (unwrap) data using a wrapkey.
    pub fn unwrap_data<T>(
        &mut self,
        key_id: ObjectId,
        data: T,
    ) -> Result<UnwrapDataResponse, SessionError>
    where T: Into<Vec<u8>>,
    {
        self.send_encrypted_command(UnwrapDataCommand {
            key_id,
            data: data.into(),
        })
    }

    /// Compute a ECDSA signature of the specified key
    pub fn sign_data_ecdsa<T>(
        &mut self,
        key_id: ObjectId,
        data: T,
    ) -> Result<SignDataECDSAResponse, SessionError>
    where
        T: Into<Vec<u8>>,
    {
        self.send_encrypted_command(SignDataECDSACommand {
            key_id,
            data: data.into(),
        })
    }

    /// Generate wrapkey
    pub fn generate_wrap_key(
        &mut self,
        key_id: ObjectId,
        label: ObjectLabel,
        domains: Domains,
        capabilities: Capabilities,
        algorithm: Algorithm,
        dc: Capabilities,
    ) -> Result<GenerateWrapKeyResponse, SessionError>
    {
        self.send_encrypted_command(GenerateWrapKeyCommand {
            key_id,
            label,
            domains,
            capabilities,
            algorithm,
            dc,
        })
    }

    /// Attest an Asymmetric key.
    pub fn attest_asymmetric(
        &mut self,
        key_id: ObjectId,
        attest_key_id: ObjectId,
    ) -> Result<AttestAsymmetricResponse, SessionError>
    {
        self.send_encrypted_command(AttestAsymmetricCommand {
            key_id,
            attest_key_id,
        })
    }

    /// Get an object in encrypted form (wrapped)
    pub fn export_wrapped(
        &mut self,
        wrapkey_id: ObjectId,
        object_type: ObjectType,
        object_id: ObjectId,
    ) -> Result<ExportWrappedResponse, SessionError>
    {
        self.send_encrypted_command(ExportWrappedCommand {
            wrapkey_id,
            object_type,
            object_id,
        })
    }

    /// Authenticate the current session with the `YubiHSM2`
    fn authenticate(&mut self) -> Result<(), SessionError> {
        let command = self.channel.authenticate_session()?;
        let response = self.send_command(command)?;
        self.channel
            .finish_authenticate_session(&response)
            .map_err(|e| e.into())
    }

    /// Send a command message to the YubiHSM2 and parse the response
    /// POST /connector/api with a given command message
    fn send_command(&mut self, cmd: CommandMessage) -> Result<ResponseMessage, SessionError> {
        let cmd_type = cmd.command_type;
        let uuid = cmd.uuid;

        // TODO: handle reconnecting when sessions are lost
        let response_bytes = self.connector.send_command(uuid, cmd.into())?;
        let response = ResponseMessage::parse(response_bytes)?;

        if response.is_err() {
            session_fail!(ResponseError, "HSM error: {:?}", response.code);
        }

        if response.command().unwrap() != cmd_type {
            session_fail!(
                ProtocolError,
                "command type mismatch: expected {:?}, got {:?}",
                cmd_type,
                response.command().unwrap()
            );
        }

        Ok(response)
    }

    /// Encrypt a command and send it to the card, then authenticate and
    /// decrypt the response
    fn send_encrypted_command<T: Command>(
        &mut self,
        command: T,
    ) -> Result<T::ResponseType, SessionError> {
        let plaintext_cmd = command.into();
        let encrypted_cmd = self.channel.encrypt_command(plaintext_cmd)?;

        let encrypted_response = self.send_command(encrypted_cmd)?;
        let response = self.channel.decrypt_response(encrypted_response)?;

        if response.is_err() {
            // TODO: factor this into ResponseMessage or ResponseCode?
            let description = match response.code {
                ResponseCode::MemoryError => "HSM memory error (missing object?)".to_owned(),
                other => format!("{:?}", other),
            };

            session_fail!(ResponseError, description);
        }

        if response.command().unwrap() != T::COMMAND_TYPE {
            session_fail!(
                ResponseError,
                "command type mismatch: expected {:?}, got {:?}",
                T::COMMAND_TYPE,
                response.command().unwrap()
            );
        }

        deserialize(response.data.as_ref()).map_err(|e| e.into())
    }
}
