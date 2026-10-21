use crate::types::DirectPluginEvent;
use reqwest::Client;

/// Error returned by `SuperPlaneClient` operations.
#[derive(Debug)]
pub struct SuperPlaneClientError {
    pub message: String,
    pub status: Option<u16>,
    pub response_body: Option<String>,
}

impl std::fmt::Display for SuperPlaneClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for SuperPlaneClientError {}

/// Client for emitting direct events into SuperPlane.
pub struct SuperPlaneClient {
    pub base_url: String,
    pub integration_id: String,
    pub token: Option<String>,
    client: Client,
}

/// Options for constructing a `SuperPlaneClient`.
pub struct SuperPlaneClientOptions {
    pub base_url: String,
    pub integration_id: String,
    pub token: Option<String>,
}

impl SuperPlaneClient {
    pub fn new(options: SuperPlaneClientOptions) -> Self {
        Self {
            base_url: options.base_url.trim_end_matches('/').to_string(),
            integration_id: options.integration_id,
            token: options.token,
            client: Client::builder()
                .use_rustls_tls()
                .build()
                .expect("failed to create HTTP client"),
        }
    }

    /// Emit a direct plugin event into SuperPlane.
    pub async fn emit_event(&self, event: &DirectPluginEvent) -> Result<(), SuperPlaneClientError> {
        let url = format!(
            "{}/api/v1/integrations/{}/events",
            self.base_url, self.integration_id
        );

        let mut req = self
            .client
            .post(&url)
            .header("content-type", "application/json");

        if let Some(token) = &self.token {
            req = req.header("authorization", format!("Bearer {}", token));
        }

        let response = req
            .json(event)
            .send()
            .await
            .map_err(|e| SuperPlaneClientError {
                message: format!("Request failed: {}", e),
                status: None,
                response_body: None,
            })?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            return Err(SuperPlaneClientError {
                message: format!("SuperPlane event request failed with {}", status),
                status: Some(status),
                response_body: Some(body),
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tls_tests {
    #[tokio::test]
    #[ignore = "requires public TLS test hosts"]
    async fn patched_verifier_survives_native_tls_feature_unification() {
        let sdk = super::SuperPlaneClient::new(super::SuperPlaneClientOptions {
            base_url: "https://example.com".into(),
            integration_id: String::new(),
            token: None,
        });
        for host in [
            "dv-g3b-emudhra.we-have-a-problem.com",
            "dv-g3b-aaa-emudhra.we-have-a-problem.com",
        ] {
            let mut rejected = false;
            for attempt in 1..=5 {
                let error = sdk
                    .client
                    .get(format!("https://{host}/"))
                    .timeout(std::time::Duration::from_secs(20))
                    .send()
                    .await
                    .expect_err("compromised chain accepted");
                let detail = format!("{error:?}");
                println!("{host} attempt {attempt}: {detail}");
                if detail.contains("CertDistrusted") {
                    rejected = true;
                    break;
                }
            }
            assert!(
                rejected,
                "no Rustls certificate rejection; network errors are inconclusive"
            );
        }
        let response = sdk
            .client
            .get("https://example.com/")
            .timeout(std::time::Duration::from_secs(20))
            .send()
            .await
            .unwrap();
        assert!(response.status().is_success());
        println!("example.com: {}", response.status());
    }
}
