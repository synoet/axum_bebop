use async_trait::async_trait;
use axum::{
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::IntoResponse,
};

type BebopRejection = (StatusCode, String);

const CONTENT_TYPE_HEADER: &str = "Content-Type";
const CONTENT_TYPE_OCTET_STREAM: &str = "application/octet-stream";

fn octet_content_type(header_map: &HeaderMap) -> bool {
    header_map
        .get(CONTENT_TYPE_HEADER)
        .map(|value| value == CONTENT_TYPE_OCTET_STREAM)
        .unwrap_or(false)
}

pub struct Bebop<T: bebop::OwnedRecord>(pub T);

#[async_trait]
impl<T, S> axum::extract::FromRequest<S> for Bebop<T>
where
    T: bebop::OwnedRecord + Send + Sync,
    S: Send + Sync,
{
    type Rejection = BebopRejection;

    async fn from_request(req: axum::extract::Request, state: &S) -> Result<Self, Self::Rejection> {
        if !octet_content_type(req.headers()) {
            return Err((
                axum::http::StatusCode::UNSUPPORTED_MEDIA_TYPE,
                "Bebop only supports octet streams".to_string(),
            ));
        }

        let bytes = axum::body::Bytes::from_request(req, state)
            .await
            .map_err(|_| {
                (
                    axum::http::StatusCode::BAD_REQUEST,
                    "body should be octet stream".to_string(),
                )
            })?;

        let bytes = bytes.to_vec().clone();

        let value = T::deserialize(bytes.as_slice()).map_err(|e| {
            (
                axum::http::StatusCode::BAD_REQUEST,
                format!("Bebop deserialization error: {:?}", e),
            )
        })?;

        Ok(Bebop(value))
    }
}

impl<T> IntoResponse for Bebop<T>
where
    T: bebop::OwnedRecord + Send + Sync,
{
    fn into_response(self) -> axum::response::Response {
        let mut buffer = Vec::with_capacity(self.0.serialized_size());
        match self.0.serialize(&mut buffer) {
            Ok(_) => (
                [(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static(CONTENT_TYPE_OCTET_STREAM),
                )],
                buffer,
            )
                .into_response(),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed to serialize bebop record: {:?}", e),
            )
                .into_response(),
        }
    }
}
