use tokio::io;
use crate::types::TokenInfo;
use crate::Error;
use hyper::header::HeaderName;
use serde::{Deserialize, Serialize};
use url::Url;

pub struct MetadataApiFlowOpts {
}
/// ServiceAccountFlow can fetch oauth tokens using a service account.
pub struct MetadataApiFlow {
}

impl MetadataApiFlow {
    pub(crate) fn new(opts: MetadataApiFlowOpts) -> Result<Self, io::Error> {
        return Ok(MetadataApiFlow{})
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
        let request = hyper::Request::get( url.as_str())
            .header(HeaderName::from_lowercase(b"metadata-flavor").unwrap(), "Google")
            .body(hyper::body::Body::empty())
            .unwrap();
        let (head, body) = hyper_client.request(request).await?.into_parts();
        let body = hyper::body::to_bytes(body).await?;
        TokenInfo::from_json(&body)
    }
}
