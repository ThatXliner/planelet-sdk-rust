use planelet_sdk::{
    ActionDefinition, ExecuteActionResponse, HandleTriggerWebhookResponse, ParameterManifest,
    Plugin, SetupTriggerResponse, CleanupTriggerResponse, TriggerDefinition,
};
use planelet_sdk::utils::decode_raw_body_text;
use serde_json::json;
use std::collections::HashMap;

#[tokio::main]
async fn main() {
    let plugin = Plugin::builder()
        .id("demo")
        .label("Demo Plugin")
        .description("A demo Planelet plugin in Rust.")
        .action(ActionDefinition {
            id: "echo".into(),
            label: "Echo".into(),
            description: Some("Returns the configured message.".into()),
            parameters: vec![ParameterManifest {
                id: "message".into(),
                label: "Message".into(),
                param_type: "string".into(),
                required: Some(true),
                ..Default::default()
            }],
            execute: Box::new(|req| {
                Box::pin(async move {
                    ExecuteActionResponse::success(
                        json!({ "message": req.parameters.get("message") }),
                    )
                })
            }),
            ..Default::default()
        })
        .trigger(TriggerDefinition {
            id: "webhook".into(),
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
                    let body = decode_raw_body_text(&req.request.raw_body_base64);
                    HandleTriggerWebhookResponse::emit(
                        "webhook.received",
                        json!({
                            "method": req.request.method,
                            "body": body,
                        }),
                        None,
                    )
                })
            })),
            cleanup: Some(Box::new(|_req| {
                Box::pin(async move { CleanupTriggerResponse::success() })
            })),
            ..Default::default()
        })
        .build();

    plugin.listen("0.0.0.0:3000").await;
}
