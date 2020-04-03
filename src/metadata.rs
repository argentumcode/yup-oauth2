use tokio::io;
use crate::types::TokenInfo;
use crate::Error;
use hyper::header::{HeaderName, HeaderValue};
use serde::{Deserialize, Serialize};
use url::Url;
use std::time::Duration;
use http::response::Parts;
use hyper::{Body, StatusCode};

pub struct MetadataApiFlowOpts {}

/// ServiceAccountFlow can fetch oauth tokens using a service account.
pub struct MetadataApiFlow {}

impl MetadataApiFlow {
    pub(crate) fn new(opts: MetadataApiFlowOpts) -> Result<Self, io::Error> {
        return Ok(MetadataApiFlow {});
    }

    pub(crate) async fn on_gce_once<C>(hyper_client: &hyper::Client<C>) -> Result<bool, Error> where
        C: hyper::client::connect::Connect + Clone + Send + Sync + 'static, {
        let request = hyper::Request::get("http://169.254.169.254")
            .header(HeaderName::from_lowercase(b"metadata-flavor").unwrap(), "Google")
            .body(hyper::body::Body::empty())
            .unwrap();
        let res = tokio::time::timeout(Duration::from_secs(3), hyper_client.request(request)).await;
        if let Ok(res) = res {
            let (head, _): (Parts, Body) = res?.into_parts();
            Ok(head.status == StatusCode::OK && head.headers.get(HeaderName::from_lowercase(b"metadata-flavor").unwrap()).unwrap_or( &HeaderValue::from_static("")) == "Google")
        } else {
            Ok(false)
        }
    }

    pub(crate) async fn on_gce<C>(hyper_client: &hyper::Client<C>) -> bool where
        C: hyper::client::connect::Connect + Clone + Send + Sync + 'static, {
        for _ in 0..3 {
            if let Ok(ret) = Self::on_gce_once(hyper_client).await {
                return ret
            }
        }
        false
    }

    /// Send a request for a new Bearer token to the OAuth provider.
    pub(crate) async fn token<C, T>(
        &self,
        hyper_client: &hyper::Client<C>,
        scopes: &[T],
    ) -> Result<TokenInfo, Error>
        where
            T: AsRef<str>,
            C: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
    {
        let mut url = Url::parse("http://metadata.google.internal/computeMetadata/v1/instance/service-accounts/default/token?scopes=").unwrap();
        if !scopes.is_empty() {
            url.query_pairs_mut().append_pair("scopes", &crate::helper::join(scopes, ","));
        }
        let request = hyper::Request::get(url.as_str())
            .header(HeaderName::from_lowercase(b"metadata-flavor").unwrap(), "Google")
            .body(hyper::body::Body::empty())
            .unwrap();
        let (_, body) = hyper_client.request(request).await?.into_parts();
        let body = hyper::body::to_bytes(body).await?;
        TokenInfo::from_json(&body)
    }
}
