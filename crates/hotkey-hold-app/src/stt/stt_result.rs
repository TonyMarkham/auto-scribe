use crate::stt::SttError;

pub(crate) type SttResult<T> = std::result::Result<T, SttError>;
