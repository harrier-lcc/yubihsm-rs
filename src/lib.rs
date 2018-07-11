//! yubihsm.rs: pure Rust client for `YubiHSM2` hardware security modules
//!
//! ## Prerequisites
//!
//! This crate builds on Rust 1.27+ and by default uses SIMD features
//! which require the following `RUSTFLAGS`:
//!
//! `RUSTFLAGS=-Ctarget-feature=+aes`
//!
//! You can configure your `~/.cargo/config` to always pass these flags:
//!
//! ```toml
//! [build]
//! rustflags = ["-Ctarget-feature=+aes"]
//! ```
//!
//! # Getting Started
//!
//! The following documentation describes the most important parts of this crate's API:
//!
//! * [Session]: end-to-end encrypted connection with the YubiHSM. You'll need an active one to do anything.
//! * [commands]: commands supported by the YubiHSM2 (i.e. main functionality)
//!
//! [Session]: https://docs.rs/yubihsm/latest/yubihsm/session/struct.Session.html
//! [commands]: https://docs.rs/yubihsm/latest/yubihsm/commands/index.html
//!
//! The following is an example of how to create a `Session` by connecting to a
//! [yubihsm-connector] process, and then performing an Ed25519 signature:
//!
//! [yubihsm-connector]: https://developers.yubico.com/YubiHSM2/Component_Reference/yubihsm-connector/
//!
//! ```no_run
//! extern crate yubihsm;
//! use yubihsm::Session;
//!
//! // Default host, port, auth key ID, and password for yubihsm-connector
//! let mut session =
//!     Session::create_from_password(Default::default(), 1, "password", true).unwrap();
//!
//! // Note: You'll need to create this key first. Run the following from yubihsm-shell:
//! // `generate asymmetric 0 100 ed25519_test_key 1 asymmetric_sign_eddsa ed25519`
//! let signature = yubihsm::sign_ed25519(&mut session, 100, "Hello, world!").unwrap();
//! println!("Ed25519 signature: {:?}", signature);
//! ```

#![crate_name = "yubihsm"]
#![crate_type = "rlib"]
#![deny(warnings, missing_docs, trivial_casts, trivial_numeric_casts)]
#![deny(unsafe_code, unused_import_braces, unused_qualifications)]
#![doc(html_root_url = "https://docs.rs/yubihsm/0.12.0-alpha1")]

extern crate aes;
#[macro_use]
extern crate bitflags;
extern crate block_modes;
extern crate byteorder;
extern crate clear_on_drop;
extern crate cmac;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate failure_derive;
#[cfg(feature = "hmac")]
extern crate hmac;
#[cfg(feature = "pbkdf2")]
extern crate pbkdf2;
extern crate rand;
#[cfg(feature = "ring")]
extern crate ring;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[cfg(feature = "sha2")]
extern crate sha2;
extern crate subtle;
#[cfg(feature = "mockhsm")]
extern crate untrusted;
extern crate uuid;

/// Error types
pub mod error;

/// Cryptographic algorithms supported by the `YubiHSM2`
pub mod algorithm;

/// Object attributes specifying which operations are allowed to be performed
pub mod capability;

/// Commands supported by the `YubiHSM`.
///
/// Functions defined in the `yubihsm::commands` module are reimported
/// and available from the toplevel `yubihsm` module as well.
///
/// For more information, see:
/// <https://developers.yubico.com/YubiHSM2/Commands/>
pub mod commands;

/// Client for the `yubihsm-connector` service
pub mod connector;

/// Logical partitions within the `YubiHSM`, allowing several applications to share the device
pub mod domain;

#[cfg(feature = "mockhsm")]
/// Software simulation of the `YubiHSM2` for integration testing,
pub mod mockhsm;

/// Objects stored in the `YubiHSM2`
///
/// For more information, see:
/// <https://developers.yubico.com/YubiHSM2/Concepts/Object.html>
pub mod object;

/// Encrypted communication channel to the YubiHSM hardware
mod securechannel;

/// Serde-powered serializers for the `YubiHSM` wire format
mod serializers;

/// `YubiHSM2` sessions: primary API for performing HSM operations
///
/// See <https://developers.yubico.com/YubiHSM2/Concepts/Session.html>
pub mod session;

pub use algorithm::{
    Algorithm, AsymmetricAlgorithm, AuthAlgorithm, HMACAlgorithm, OTPAlgorithm, OpaqueAlgorithm,
    WrapAlgorithm,
};
pub use capability::Capability;
pub use commands::*;
pub use connector::{Connector, HttpConfig, HttpConnector};
pub use domain::Domain;
pub use object::Id as ObjectId;
pub use object::Label as ObjectLabel;
pub use object::Origin as ObjectOrigin;
pub use object::SequenceId;
pub use object::Type as ObjectType;
pub use securechannel::{SessionId, StaticKeys};
pub use session::{Session, SessionError};
