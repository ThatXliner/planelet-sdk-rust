# Planelet SDK for Rust

Build Planelet plugin servers that SuperPlane can call directly. Uses `axum` for HTTP, `serde` for serialization, and `reqwest` for the SuperPlane client.

## Install

Add to your `Cargo.toml`:

```toml
[dependencies]
planelet-sdk = { path = "../planelet-sdk-rust" }
tokio = { version = "1", features = ["full"] }
serde_json = "1"
```

## Quick start

```rust
use planelet_sdk::{
    Plugin, ActionDefinition, ParameterManifest,
    ExecuteActionRequest, ExecuteActionResponse,
};
use serde_json::json;

#[tokio::main]
async fn main() {
    let plugin = Plugin::builder()
        .id("demo")
        .label("Demo Plugin")
        .action(ActionDefinition {
            id: "echo".into(),
            label: "Echo".into(),
            parameters: vec![
                ParameterManifest {
                    id: "message".into(),
                    label: "Message".into(),
                    param_type: "string".into(),
                    required: Some(true),
                    ..Default::default()
                },
            ],
            execute: Box::new(|req| {
                Box::pin(async move {
                    ExecuteActionResponse::success(
                        json!({ "message": req.parameters["message"] }),
                    )
                })
            }),
            ..Default::default()
        })
        .build();

    plugin.listen("0.0.0.0:3000").await;
}
```

Point the Planelet integration in SuperPlane at `http://host.docker.internal:3000` when SuperPlane is running in Docker, or at the reachable host/port for your environment.

## Webhook trigger

```rust
use planelet_sdk::{
    Plugin, TriggerDefinition, HandleTriggerWebhookResponse,
    SetupTriggerResponse, CleanupTriggerResponse,
};
use planelet_sdk::utils::{decode_raw_body_text, first_header};
use serde_json::json;
use std::collections::HashMap;

let plugin = Plugin::builder()
    .id("webhooks")
    .label("Webhooks")
    .trigger(TriggerDefinition {
        id: "incoming".into(),
        label: "Incoming Webhook".into(),
        parameters: vec![],
        setup: Some(Box::new(|req| {
            Box::pin(async move {
                let mut meta = HashMap::new();
                meta.insert(
                    "webhookUrl".into(),
                    serde_json::Value::String(req.webhook.url),
                );
                SetupTriggerResponse::success(Some(meta))
            })
        })),
        webhook: Some(Box::new(|req| {
            Box::pin(async move {
                let sig = first_header(&req.request.headers, "x-provider-signature");
                if sig.is_none() {
                    return HandleTriggerWebhookResponse::error(
                        "Missing signature",
                        Some(401),
                    );
                }
                let body = decode_raw_body_text(&req.request.raw_body_base64);
                HandleTriggerWebhookResponse::emit(
                    "incoming.received",
                    serde_json::from_str(&body).unwrap_or(json!(body)),
                    None,
                )
            })
        })),
        cleanup: Some(Box::new(|_| {
            Box::pin(async { CleanupTriggerResponse::success() })
        })),
        ..Default::default()
    })
    .build();
```

Use `metadata` for provider webhook IDs or setup state. Do not put secrets in the manifest.

## Direct events

Plugins can emit events into SuperPlane without waiting for a third-party webhook:

```rust
use planelet_sdk::{SuperPlaneClient, SuperPlaneClientOptions, DirectPluginEvent};
use serde_json::json;

let client = SuperPlaneClient::new(SuperPlaneClientOptions {
    base_url: "https://superplane.example".into(),
    integration_id: "integration-id".into(),
    token: Some("my-token".into()),
});

client.emit_event(&DirectPluginEvent {
    event_type: "build.finished".into(),
    payload: json!({ "status": "passed" }),
}).await.unwrap();
```

## Using with axum

If you want to mount the plugin under an existing axum router or add middleware:

```rust
let router = plugin.into_router();
// You can nest it, add layers, etc.
```

## Utilities

- `decode_raw_body(base64_str)` - Decode a base64-encoded webhook body to `Vec<u8>`
- `decode_raw_body_text(base64_str)` - Decode a base64-encoded webhook body to `String`
- `first_header(headers, name)` - Get the first value of a header (case-insensitive)

## Plugin API

See the [TypeScript SDK README](../planelet-sdk-ts/README.md) for the full protocol specification. This Rust SDK implements the same wire protocol with camelCase JSON field names.
