// Tako -- Take container image.
// Copyright 2018 Arian van Putten, Ruud van Asseldonk, Tako Marks.

//! Errors that Tako can encounter.

use std::io;
use std::result;

use base64;

#[derive(Debug)]
pub enum Error {
    /// Error in config file on a given line.
    InvalidConfig(usize, &'static str),

    /// A key is missing in the config.
    IncompleteConfig(&'static str),

    /// Public key in config could not be parsed as base64.
    InvalidPublicKeyData(usize, base64::DecodeError),

    /// Error in manifest file.
    InvalidManifest(&'static str),

    /// Signature in manifest could not be parsed as base64.
    InvalidSignatureData(base64::DecodeError),

    /// Signature verification failed.
    InvalidSignature,

    /// IO error.
    IoError(io::Error),
}

pub type Result<T> = result::Result<T, Error>;

// TODO: Implement std::error::Error for Error.
