use std::str::FromStr;
use std::error::Error;
use std::fmt;

pub struct AuthResult {
    pub identity: Vec<u8>,
    pub error: bool,
}

pub struct AuthSpec {
    pub method: String,
    pub key: String,
    pub signature: String,
}

impl AuthSpec {
    pub fn valid(&self) -> bool {
        self.key.len() > 0
    }
}

impl AuthResult {
    pub fn active(&self) -> bool {
        self.identity.len() > 0
    }

    pub fn valid(&self) -> bool {
        !self.error
    }
}

impl fmt::Debug for AuthResult {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.write_str(&hex::encode(&self.identity))
    }
}

#[derive(Debug)]
pub struct AuthSpecError;

impl Error for AuthSpecError {
    fn description(&self) -> &str{
        "auth string malformed"
    }
}

impl fmt::Display for AuthSpecError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.write_str(self.description())
    }
}


impl FromStr for AuthSpec {
    type Err = AuthSpecError;

    fn from_str(s: &str) -> Result<AuthSpec, AuthSpecError> {
        let mut auth_kv = s.split(" ");
        match auth_kv.next() {
            Some(v) => {
                if v != "PUBSIG" {
                    return Err(AuthSpecError{});
                }
            },
            _ => {},
        };

        let ss = match auth_kv.next() {
            Some(v) => {
                v
            },
            _ => {
                return Err(AuthSpecError{});
            },
        };

        let mut auth_fields = ss.split(":");
        if auth_fields.clone().count() != 3 {
            return Err(AuthSpecError{})
        }
        let auth_type: String = auth_fields.next().unwrap().to_string();
        let auth_key: String = auth_fields.next().unwrap().to_string();
        let auth_signature: String = auth_fields.next().unwrap().to_string();

        let r = AuthSpec{
            method: auth_type,
            key: auth_key,
            signature: auth_signature,
        };
        Ok(r)
    }
}

impl fmt::Debug for AuthSpec {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.write_str(
            format!(
                "{} key {:?}",
                self.method,
                self.key,
                ).as_str()
            )
    }
}

#[derive(Debug)]
pub struct AuthError;

impl fmt::Display for AuthError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.write_str(self.description())
    }
}

impl Error for AuthError {
    fn description(&self) -> &str{
        "auth key signature mismatch"
    }
}


#[cfg(feature = "dev")]
pub mod mock;

#[cfg(feature = "pgpauth")]
pub mod pgp;
