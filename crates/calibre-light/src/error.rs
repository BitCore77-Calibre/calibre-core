use thiserror::Error;

#[derive(Debug, Error)]
pub enum LightError {
    #[error("rpc transport: {0}")]
    Transport(#[from] reqwest::Error),
    #[error("rpc error {code}: {message}")]
    Rpc { code: i64, message: String },
    #[error("malformed response: {0}")]
    Parse(String),
    #[error("proof root not in known_roots")]
    UnknownRoot,
    #[error("merkle path did not fold to claimed root")]
    BadPath,
    #[error("hex: {0}")]
    Hex(String),
}
