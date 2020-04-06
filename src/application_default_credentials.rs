//! default
use std::path::Path;

use hyper::client::HttpConnector;
use log::MetadataBuilder;

use crate::{Credentials, MetadataApiAuthenticator, read_credential_file, read_user_credentials, ServiceAccountAuthenticator, UserCredentialsAuthenticator};
use crate::authenticator::Authenticator;
use crate::Credentials::UserCredentials;
use crate::metadata::MetadataApiFlow;

/// def
pub async fn application_default_credentials() -> Result<Authenticator<hyper_rustls::HttpsConnector<hyper::client::connect::HttpConnector>>, std::io::Error> {
    if let Ok(path) = std::env::var("GOOGLE_APPLICATION_CREDENTIALS") {
        let cred = read_credential_file(path).await?;
        return match cred {
            Credentials::ServiceAccountKey(c) => {
                ServiceAccountAuthenticator::builder(c).build().await
            },
            Credentials::UserCredentials(c) => {
                UserCredentialsAuthenticator::builder(c).build().await
            },
        }
    }
    if let Some(home_dir) = dirs::home_dir() {
        // TODO: This is not correct for Windows
        let config_path = home_dir.join(Path::new(".config")).join(Path::new("gcloud")).join(Path::new("application_default_credentials.json"));
        if config_path.exists() {
            let cred = read_credential_file(config_path).await?;
            return match cred {
                Credentials::ServiceAccountKey(c) => {
                    ServiceAccountAuthenticator::builder(c).build().await
                },
                Credentials::UserCredentials(c) => {
                    UserCredentialsAuthenticator::builder(c).build().await
                },
            }
        }
    }

    let hyper_client = hyper::Client::new();
    if MetadataApiFlow::on_gce(&hyper_client).await {
        return MetadataApiAuthenticator::builder().build().await
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        format!("could not find default credentials"),
    ))
}