use crate::StatusCode;
use axum::response::{IntoResponse, Response};

pub(crate) struct RgError(eyre::Error, StatusCode);

pub(crate) trait WithStatusCode {
    type Output;
    fn with_status_code(self, code: StatusCode) -> Self::Output;
}

impl IntoResponse for RgError {
    fn into_response(self) -> Response {
        (self.1, self.0.to_string()).into_response()
    }
}

impl WithStatusCode for RgError {
    type Output = RgError;
    fn with_status_code(self, code: StatusCode) -> Self::Output {
        Self(self.0, code)
    }
}

impl<E> From<E> for RgError
where
    E: Into<eyre::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into(), StatusCode::INTERNAL_SERVER_ERROR)
    }
}

impl<T, E> WithStatusCode for Result<T, E>
where
    E: Into<eyre::Error>,
{
    type Output = Result<T, RgError>;
    fn with_status_code(self, code: StatusCode) -> Self::Output {
        self.map_err(|e| RgError::from(e).with_status_code(code))
    }
}
