use crate::error::AppError;

pub(crate) type AppResult<T> = std::result::Result<T, AppError>;
