use crate::error::{AppError, AppResult};

use std::fmt::Display;

pub(crate) trait ResultContext<T> {
    fn context(self, context: impl Display) -> AppResult<T>;

    fn with_context(self, context: impl FnOnce() -> String) -> AppResult<T>;
}

impl<T, E> ResultContext<T> for Result<T, E>
where
    E: Display,
{
    #[track_caller]
    fn context(self, context: impl Display) -> AppResult<T> {
        self.map_err(|error| AppError::with_context(context, error))
    }

    #[track_caller]
    fn with_context(self, context: impl FnOnce() -> String) -> AppResult<T> {
        self.map_err(|error| AppError::with_context(context(), error))
    }
}
