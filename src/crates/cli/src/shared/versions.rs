use serde::de::DeserializeOwned;
use serde::Deserialize;
use std::error::Error;
use std::future::Future;
use std::pin::Pin;

const GO_DOWNLOAD_URL: &str = "https://go.dev/dl/?mode=json&include=all";

type FetchError = Box<dyn Error + Send + Sync>;
type FetchResult<T> = Result<T, FetchError>;

/// Represents version information fetched from the Go download server.
#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct GoVersionInfo {
    pub version: String,
    pub stable: bool,
}

/// Thin abstraction to allow swapping the HTTP client for tests.
pub trait VersionHttpClient {
    fn get_json<'a, T>(
        &'a self,
        url: &'a str,
    ) -> Pin<Box<dyn Future<Output = FetchResult<T>> + Send + 'a>>
    where
        T: DeserializeOwned + Send + 'static;
}

#[derive(Default)]
struct ReqwestHttpClient {
    client: reqwest::Client,
}

impl VersionHttpClient for ReqwestHttpClient {
    fn get_json<'a, T>(
        &'a self,
        url: &'a str,
    ) -> Pin<Box<dyn Future<Output = FetchResult<T>> + Send + 'a>>
    where
        T: DeserializeOwned + Send + 'static,
    {
        Box::pin(async move {
            let response = self.client.get(url).send().await.map_err(to_fetch_error)?;
            let parsed = response
                .json::<T>()
                .await
                .map_err(to_fetch_error)?;
            Ok(parsed)
        })
    }
}

/// Fetches the list of available Go versions from the official Go website.
///
/// This function queries the JSON endpoint that includes all historical versions.
pub async fn fetch_remote_versions() -> FetchResult<Vec<GoVersionInfo>> {
    let client = ReqwestHttpClient::default();
    fetch_remote_versions_with_client(&client).await
}

/// Same as `fetch_remote_versions` but allows injecting an HTTP client for testing.
pub async fn fetch_remote_versions_with_client<C: VersionHttpClient>(
    client: &C,
) -> FetchResult<Vec<GoVersionInfo>> {
    client.get_json(GO_DOWNLOAD_URL).await
}

fn to_fetch_error<E>(err: E) -> FetchError
where
    E: Error + Send + Sync + 'static,
{
    Box::new(err)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    struct StubClient {
        versions: Vec<GoVersionInfo>,
        requested_url: Arc<Mutex<Option<String>>>,
    }

    impl StubClient {
        fn new(versions: Vec<GoVersionInfo>) -> Self {
            Self {
                versions,
                requested_url: Arc::new(Mutex::new(None)),
            }
        }

        fn requested_url(&self) -> Option<String> {
            self.requested_url.lock().ok().and_then(|u| u.clone())
        }
    }

    impl VersionHttpClient for StubClient {
        fn get_json<'a, T>(
            &'a self,
            url: &'a str,
        ) -> Pin<Box<dyn Future<Output = FetchResult<T>> + Send + 'a>>
        where
            T: DeserializeOwned + Send + 'static,
        {
            let versions = self.versions.clone();
            let url = url.to_string();
            let requested_url = self.requested_url.clone();

            Box::pin(async move {
                if let Ok(mut slot) = requested_url.lock() {
                    *slot = Some(url);
                }
                let json = serde_json::to_string(&versions).map_err(to_fetch_error)?;
                let parsed = serde_json::from_str::<T>(&json).map_err(to_fetch_error)?;
                Ok(parsed)
            })
        }
    }

    struct ErrorClient;

    impl VersionHttpClient for ErrorClient {
        fn get_json<'a, T>(
            &'a self,
            _url: &'a str,
        ) -> Pin<Box<dyn Future<Output = FetchResult<T>> + Send + 'a>>
        where
            T: DeserializeOwned + Send + 'static,
        {
            Box::pin(async move { Err("network error".into()) })
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn fetches_versions_via_injected_client() {
        let expected = vec![
            GoVersionInfo {
                version: "go1.22.0".to_string(),
                stable: true,
            },
            GoVersionInfo {
                version: "go1.21.5".to_string(),
                stable: true,
            },
        ];
        let client = StubClient::new(expected.clone());

        let versions = fetch_remote_versions_with_client(&client).await.unwrap();

        assert_eq!(versions, expected);
        assert_eq!(
            client.requested_url().as_deref(),
            Some(GO_DOWNLOAD_URL),
            "should call the Go download endpoint"
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn bubbles_up_http_errors() {
        let err = fetch_remote_versions_with_client(&ErrorClient)
            .await
            .unwrap_err();

        assert_eq!(err.to_string(), "network error");
    }
}
