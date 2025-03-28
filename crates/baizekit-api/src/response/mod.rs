mod err;
mod msg;
mod ok;

pub use err::*;
pub use msg::*;
pub use ok::*;

pub type Result<T, E> = std::result::Result<T, ApiError<E>>;
pub type ApiResult<T, E> = std::result::Result<ApiOK<T>, ApiError<E>>;
