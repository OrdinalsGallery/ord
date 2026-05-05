use super::*;

#[derive(Default)]
pub struct ServerConfig {
  pub accept_offers: bool,
  pub chain: Chain,
  pub csp_origin: Option<String>,
  pub decompress: bool,
  pub domain: Option<String>,
  pub index_sats: bool,
  pub json_api_enabled: bool,
  pub proxy: Option<Url>,
}

impl ServerConfig {
  pub(super) fn preview_content_security_policy(
    &self,
    media: Media,
    host: Option<&str>,
  ) -> ServerResult<[(HeaderName, HeaderValue); 1]> {
    let default = match media {
      Media::Audio => {
        "default-src 'self'; script-src 'self' 'unsafe-inline' 'wasm-unsafe-eval'; style-src 'self' 'unsafe-inline'; media-src 'self' blob:; connect-src 'self'"
      }
      Media::Code(_) => "script-src-elem 'self' https://cdn.jsdelivr.net",
      Media::Font => "script-src-elem 'self'; style-src 'self' 'unsafe-inline'",
      Media::Iframe => {
        return Err(
          anyhow!("preview_content_security_policy cannot be called with Media::Iframe").into(),
        );
      }
      Media::Image(_) => "default-src 'self' 'unsafe-inline'",
      Media::Markdown => "script-src-elem 'self' https://cdn.jsdelivr.net",
      Media::Model => "script-src-elem 'self' https://ajax.googleapis.com",
      Media::Pdf => "script-src-elem 'self' https://cdn.jsdelivr.net",
      Media::Text => "default-src 'self'",
      Media::Unknown => "default-src 'self'",
      Media::Video => "default-src 'self'",
    };

    let value = if let Some(origin) = self.csp_origin.as_deref().or(host) {
      default
        .replace("'self'", origin)
        .parse()
        .map_err(|err| anyhow!("invalid content-security-policy origin `{origin}`: {err}"))?
    } else {
      HeaderValue::from_static(default)
    };

    Ok([(header::CONTENT_SECURITY_POLICY, value)])
  }

  pub(super) fn embed_content_security_policy(
    &self,
    media: Media,
    host: Option<&str>,
  ) -> ServerResult<[(HeaderName, HeaderValue); 1]> {
    let base = match media {
      Media::Audio => {
        "default-src 'self'; script-src 'self' 'unsafe-inline' 'wasm-unsafe-eval'; \
         style-src 'self' 'unsafe-inline'; media-src 'self' blob:; \
         connect-src 'self'; img-src 'self' data:"
      }
      Media::Image(_) => {
        "default-src 'self'; img-src 'self' data:; style-src 'self' 'unsafe-inline'"
      }
      Media::Video => {
        "default-src 'self'; media-src 'self' blob:; style-src 'self' 'unsafe-inline'"
      }
      _ => "default-src 'self'; style-src 'self' 'unsafe-inline'",
    };

    let scoped = if let Some(origin) = self.csp_origin.as_deref().or(host) {
      base.replace("'self'", origin)
    } else {
      base.to_string()
    };

    let value = format!("{scoped}; frame-ancestors *")
      .parse()
      .map_err(|err| anyhow!("invalid embed content-security-policy: {err}"))?;

    Ok([(header::CONTENT_SECURITY_POLICY, value)])
  }

  pub(super) fn public_origin(&self, headers: &HeaderMap) -> String {
    if let Some(origin) = &self.csp_origin {
      return origin.clone();
    }

    if let Some(domain) = &self.domain {
      return format!("https://{domain}");
    }

    let host = headers
      .get(header::HOST)
      .and_then(|h| h.to_str().ok());

    if let Some(host) = host {
      let scheme = headers
        .get("x-forwarded-proto")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("http");
      return format!("{scheme}://{host}");
    }

    "http://localhost".into()
  }
}
