use thiserror::Error;

#[derive(Debug, Error)]
pub enum HttpError {
    #[error("")]
    ProtocolError,
    #[error("")]
    MethodError,
    #[error("")]
    UrlError,
    #[error("")]
    HeaderError,
}
