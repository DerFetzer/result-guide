use crate::StatusCode;
use axum::response::{IntoResponse, Response};

pub(crate) struct RgError(eyre::Error, StatusCode);

impl IntoResponse for RgError {
    fn into_response(self) -> Response {
        (self.1, self.0.to_string()).into_response()
    }
}

impl RgError {
    pub fn with_status_code(self, code: StatusCode) -> Self {
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
