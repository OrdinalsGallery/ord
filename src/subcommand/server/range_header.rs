use super::*;

#[derive(Default, Debug)]
pub(crate) struct RangeHeader(pub(crate) Option<String>);

impl<S> axum::extract::FromRequestParts<S> for RangeHeader
where
  S: Send + Sync,
{
  type Rejection = (StatusCode, &'static str);

  async fn from_request_parts(
    parts: &mut http::request::Parts,
    _state: &S,
  ) -> Result<Self, Self::Rejection> {
    Ok(Self(
      parts
        .headers
        .get("range")
        .and_then(|value| value.to_str().ok())
        .map(|s| s.to_owned()),
    ))
  }
}
