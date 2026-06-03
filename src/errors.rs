use quasar_lang::prelude::*;

#[error_code]
pub enum ElectionError {
    Unauthorized,
    InvalidCandidate,
}
