use axum::response::IntoResponse;
use thiserror::Error;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum AppError {
    #[error("Not found: {resource} with id {id}")]
    NotFound { resource: String, id: String },
    #[error("Conflict: {resource} with id {id} already exists")]
    Conflict { resource: String, id: String },
    #[error("Database error: {0}")]
    DbError(#[from] diesel::result::Error),
    #[error("Connection pool error: {0}")]
    PoolError(#[from] deadpool_diesel::PoolError),
    #[error("Connection interact error: {0}")]
    InteractError(#[from] deadpool_diesel::InteractError),
    #[error("Email error: {0}")]
    EmailError(#[from] lettre::error::Error),
    #[error("Email send error: {0}")]
    EmailSendError(#[from] lettre::transport::smtp::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl AppError {
    pub fn not_found(resource: impl ToString, id: impl ToString) -> Self {
        Self::NotFound {
            resource: resource.to_string(),
            id: id.to_string(),
        }
    }

    pub fn conflict(resource: impl ToString, id: impl ToString) -> Self {
        Self::Conflict {
            resource: resource.to_string(),
            id: id.to_string(),
        }
    }

    pub fn other(error: impl Into<anyhow::Error>) -> Self {
        Self::Other(error.into())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::http::Response<axum::body::Body> {
        let status_code = match &self {
            Self::NotFound { .. } => axum::http::StatusCode::NOT_FOUND,
            Self::Conflict { .. } => axum::http::StatusCode::CONFLICT,
            _ => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        };

        (status_code, self.to_string()).into_response()
    }
}
