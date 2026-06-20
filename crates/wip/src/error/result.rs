use crate::error::WipError;

use std::result::Result as StdResult;

#[allow(unused)]
pub type WipResult<T> = StdResult<T, WipError>;
