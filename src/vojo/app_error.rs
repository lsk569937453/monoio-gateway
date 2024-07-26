use std::error::Error as _;
use thiserror::Error;

#[derive(Clone, Debug, Eq, Error, PartialEq)]
#[error("{0}")]
pub struct AppError(pub String);
// impl From<anyhow::Error> for AppError {
//     fn from(error: anyhow::Error) -> Self {
//         AppError(error.to_string())
//     }
// }
#[macro_export]
macro_rules! ensure {
    ($cond:expr, $err:expr) => {
        if !$cond {
            return Err(AppError($err.to_string()).into());
        }
    };
}
