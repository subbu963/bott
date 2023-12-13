use crate::errors::BottError;

pub type BottResult<T> = std::result::Result<T, BottError>;
