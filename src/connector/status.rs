//! Status responses from yubihsm-connector

use super::ConnectorError;

/// Status response from yubihsm-connector containing information about its
/// health and what `YubiHSM2` we're connected to
#[derive(Clone, Debug)]
pub struct Status {
    /// Status message for yubihsm-connector e.g. "OK"
    pub message: String,

    /// Serial number of `YubiHSM2` device. Only available if yubihsm-connector
    /// has been started with the --serial option
    pub serial: Option<String>,

    /// `YubiHSM2` SDK version for yubihsm-connector
    pub version: String,

    /// PID of yubihsm-connector
    pub pid: u32,
}

impl Status {
    /// Parse the yubihsm-connector status response into a status struct
    pub fn parse(response_body: &str) -> Result<Self, ConnectorError> {
        let mut response_message: Option<&str> = None;
        let mut response_serial: Option<&str> = None;
        let mut response_version: Option<&str> = None;
        let mut response_pid: Option<&str> = None;

        for line in response_body.split('\n') {
            if line.is_empty() {
                continue;
            }

            let mut fields = line.split('=');

            let key = fields
                .next()
                .ok_or_else(|| connector_err!(ResponseError, "couldn't parse key"))?;

            let value = fields
                .next()
                .ok_or_else(|| connector_err!(ResponseError, "couldn't parse value"))?;

            if let Some(remaining) = fields.next() {
                connector_fail!(ResponseError, "unexpected additional data: {}", remaining)
            }

            match key {
                "status" => response_message = Some(value),
                "serial" => response_serial = Some(value),
                "version" => response_version = Some(value),
                "pid" => response_pid = Some(value),
                _ => (),
            }
        }

        let message = response_message
            .ok_or_else(|| connector_err!(ResponseError, "missing status"))?
            .to_owned();

        let serial = match response_serial {
            Some("*") => None,
            Some(s) => Some(s.to_owned()),
            None => connector_fail!(ResponseError, "missing serial"),
        };

        let version = response_version
            .ok_or_else(|| connector_err!(ResponseError, "missing version"))?
            .to_owned();

        let pid = response_pid
            .ok_or_else(|| connector_err!(ResponseError, "missing PID"))?
            .parse()
            .map_err(|_| connector_err!(ResponseError, "invalid PID: {}", response_pid.unwrap()))?;

        Ok(Self {
            message,
            serial,
            version,
            pid,
        })
    }
}
