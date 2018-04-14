// Tako -- Take container image.
// Copyright 2018 Arian van Putten, Ruud van Asseldonk, Tako Marks.

//! Configuration file parser.

use std::str::FromStr;
use std::path::PathBuf;

use base64;
use hyper::Uri;

use error::{Error, Result};

struct Config {
    origin: Uri,
    public_key: [u8; 32],
    destination: PathBuf,
    restart_units: Vec<String>,
}

fn parse_public_key(lineno: usize, key_base64: &str) -> Result<[u8; 32]> {
    let bytes = match base64::decode(key_base64) {
        Ok(bs) => bs,
        Err(err) => return Err(Error::InvalidPublicKey(lineno, err)),
    };

    if bytes.len() != 32 {
        let msg = "Ed25519 public key is not 32 bytes (48 characters base64).";
        return Err(Error::InvalidConfig(lineno, msg))
    }

    let mut result = [0_u8; 32];
    result.copy_from_slice(&bytes[..]);

    Ok(result)
}

impl Config {
    pub fn parse<'a, I, S>(mut lines: I) -> Result<Config>
    where I: IntoIterator<Item = S>,
          S: AsRef<str> {
        let mut origin = None;
        let mut public_key = None;
        let mut destination = None;
        let mut restart_units = Vec::new();

        for (lineno, line_raw) in lines.into_iter().enumerate() {
            let line = line_raw.as_ref();

            // Allow empty lines in the config file.
            if line.len() == 0 {
                continue
            }

            if let Some(n) = line.find('=') {
                let key = &line[..n];
                let value = &line[n + 1..];
                match key {
                    "Origin" => {
                        match Uri::from_str(value) {
                            Ok(uri) => origin = Some(uri),
                            Err(err) => return Err(Error::InvalidUri(lineno, err)),
                        }
                    }
                    "PublicKey" => {
                        public_key = Some(parse_public_key(lineno, value)?);
                    }
                    "Destination" => {
                        destination = Some(PathBuf::from(value));
                    }
                    "RestartUnit" => {
                        restart_units.push(String::from(value));
                    }
                    _ => {
                        let msg = "Unknown key. Expected 'Origin', 'PublicKey', 'Destination', or 'RestartUnit'.";
                        return Err(Error::InvalidConfig(lineno, msg))
                    }
                }
            } else {
                let msg = "Line contains no '='. Expected 'Origin=https://example.com'-like key-value pair.";
                return Err(Error::InvalidConfig(lineno, msg))
            }
        }

        let config = Config {
            origin: match origin {
                Some(o) => o,
                None => return Err(Error::IncompleteConfig("Origin not set. Expected 'Origin='-line.")),
            },
            public_key: match public_key {
                Some(k) => k,
                None => return Err(Error::IncompleteConfig("Public key not set. Expected 'PublicKey='-line.")),
            },
            destination: match destination {
                Some(d) => d,
                None => return Err(Error::IncompleteConfig("Destination not set. Expected 'Destination=/path'-line.")),
            },
            restart_units: restart_units,
        };

        Ok(config)
    }
}

#[cfg(test)]
mod test {
    use super::Config;

    #[test]
    pub fn config_with_0_restart_units_is_parsed() {
        let config_lines = [
            "Origin=https://images.example.com/app-foo",
            "PublicKey=8+r5DKNN/cwI+h0oHxMtgdyND3S/5xDLHQu0hFUmq+g=",
            "Destination=/var/lib/images/app-foo",
        ];
        let config_res = Config::parse(&config_lines);
        assert!(config_res.is_ok());
        // TODO: Assert contents.
    }

}