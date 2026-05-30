use crate::types::*;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

// ---------------------------------------------------------------------------
// Handler function types
// ---------------------------------------------------------------------------

/// Boxed async function type for action handlers.
pub type ActionHandler = Box<
    dyn Fn(ExecuteActionRequest) -> Pin<Box<dyn Future<Output = ExecuteActionResponse> + Send>>
        + Send
        + Sync,
>;

/// Boxed async function type for trigger setup handlers.
pub type TriggerSetupHandler = Box<
    dyn Fn(SetupTriggerRequest) -> Pin<Box<dyn Future<Output = SetupTriggerResponse> + Send>>
        + Send
        + Sync,
>;

/// Boxed async function type for trigger webhook handlers.
pub type TriggerWebhookHandler = Box<
    dyn Fn(
            HandleTriggerWebhookRequest,
        ) -> Pin<Box<dyn Future<Output = HandleTriggerWebhookResponse> + Send>>
        + Send
        + Sync,
>;

/// Boxed async function type for trigger cleanup handlers.
pub type TriggerCleanupHandler = Box<
    dyn Fn(CleanupTriggerRequest) -> Pin<Box<dyn Future<Output = CleanupTriggerResponse> + Send>>
        + Send
        + Sync,
>;

// ---------------------------------------------------------------------------
// Action and trigger definitions
// ---------------------------------------------------------------------------

/// Definition of a plugin action, including manifest metadata and the execute handler.
pub struct ActionDefinition {
    pub id: String,
    pub label: String,
    pub icon: Option<String>,
    pub icon_url: Option<String>,
    pub description: Option<String>,
    pub parameters: Vec<ParameterManifest>,
    pub execute: ActionHandler,
}

impl Default for ActionDefinition {
    fn default() -> Self {
        Self {
            id: String::new(),
            label: String::new(),
            icon: None,
            icon_url: None,
            description: None,
            parameters: Vec::new(),
            execute: Box::new(|_| {
                Box::pin(async { ExecuteActionResponse::error("Not implemented") })
            }),
        }
    }
}

/// Definition of a plugin trigger, including manifest metadata and lifecycle handlers.
pub struct TriggerDefinition {
    pub id: String,
    pub label: String,
    pub icon: Option<String>,
    pub icon_url: Option<String>,
    pub description: Option<String>,
    pub parameters: Vec<ParameterManifest>,
    pub setup: Option<TriggerSetupHandler>,
    pub webhook: Option<TriggerWebhookHandler>,
    pub cleanup: Option<TriggerCleanupHandler>,
}

impl Default for TriggerDefinition {
    fn default() -> Self {
        Self {
            id: String::new(),
            label: String::new(),
            icon: None,
            icon_url: None,
            description: None,
            parameters: Vec::new(),
            setup: None,
            webhook: None,
            cleanup: None,
        }
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

/// The built plugin, holding manifest data and handler maps.
pub struct Plugin {
    manifest: PluginManifest,
    actions: HashMap<String, ActionDefinition>,
    triggers: HashMap<String, TriggerDefinition>,
}

impl Plugin {
    /// Start building a new plugin.
    pub fn builder() -> PluginBuilder {
        PluginBuilder::default()
    }

    /// Get the plugin manifest.
    pub fn manifest(&self) -> &PluginManifest {
        &self.manifest
    }

    /// Convert the plugin into an `axum::Router`.
    pub fn into_router(self) -> Router {
        let shared = Arc::new(self);
        Router::new()
            .route("/manifest", get(handle_manifest))
            .route(
                "/actions/{action_id}/execute",
                post(handle_action_execute),
            )
            .route(
                "/triggers/{trigger_id}/setup",
                post(handle_trigger_setup),
            )
            .route(
                "/triggers/{trigger_id}/webhook",
                post(handle_trigger_webhook),
            )
            .route(
                "/triggers/{trigger_id}/cleanup",
                post(handle_trigger_cleanup),
            )
            .with_state(shared)
    }

    /// Convenience: build the router and start listening on the given address.
    pub async fn listen(self, addr: &str) {
        let router = self.into_router();
        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
        println!("Planelet plugin listening on {}", addr);
        axum::serve(listener, router).await.unwrap();
    }
}

// ---------------------------------------------------------------------------
// PluginBuilder
// ---------------------------------------------------------------------------

/// Builder for constructing a `Plugin`.
#[derive(Default)]
pub struct PluginBuilder {
    id: Option<String>,
    label: Option<String>,
    icon: Option<String>,
    icon_url: Option<String>,
    description: Option<String>,
    actions: Vec<ActionDefinition>,
    triggers: Vec<TriggerDefinition>,
}

impl PluginBuilder {
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn icon_url(mut self, icon_url: impl Into<String>) -> Self {
        self.icon_url = Some(icon_url.into());
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn action(mut self, action: ActionDefinition) -> Self {
        self.actions.push(action);
        self
    }

    pub fn trigger(mut self, trigger: TriggerDefinition) -> Self {
        self.triggers.push(trigger);
        self
    }

    /// Consume the builder and produce a `Plugin`.
    ///
    /// # Panics
    ///
    /// Panics if `id` or `label` have not been set, or if duplicate action/trigger IDs exist.
    pub fn build(self) -> Plugin {
        let id = self.id.expect("Plugin id is required");
        let label = self.label.expect("Plugin label is required");

        let action_manifests: Vec<ActionManifest> = self
            .actions
            .iter()
            .map(|a| ActionManifest {
                id: a.id.clone(),
                label: a.label.clone(),
                icon: a.icon.clone(),
                icon_url: a.icon_url.clone(),
                description: a.description.clone(),
                parameters: a.parameters.clone(),
            })
            .collect();

        let trigger_manifests: Vec<TriggerManifest> = self
            .triggers
            .iter()
            .map(|t| TriggerManifest {
                id: t.id.clone(),
                label: t.label.clone(),
                icon: t.icon.clone(),
                icon_url: t.icon_url.clone(),
                description: t.description.clone(),
                parameters: t.parameters.clone(),
            })
            .collect();

        let manifest = PluginManifest {
            id: id.clone(),
            label,
            icon: self.icon,
            icon_url: self.icon_url,
            description: self.description,
            actions: action_manifests,
            triggers: trigger_manifests,
        };

        let mut actions_map = HashMap::new();
        for action in self.actions {
            if actions_map.contains_key(&action.id) {
                panic!("Duplicate action id \"{}\"", action.id);
            }
            actions_map.insert(action.id.clone(), action);
        }

        let mut triggers_map = HashMap::new();
        for trigger in self.triggers {
            if triggers_map.contains_key(&trigger.id) {
                panic!("Duplicate trigger id \"{}\"", trigger.id);
            }
            triggers_map.insert(trigger.id.clone(), trigger);
        }

        Plugin {
            manifest,
            actions: actions_map,
            triggers: triggers_map,
        }
    }
}

// ---------------------------------------------------------------------------
// Axum handlers
// ---------------------------------------------------------------------------

fn json_response(status: StatusCode, body: Value) -> Response {
    let mut response = Json(body).into_response();
    *response.status_mut() = status;
    response
}

fn error_json(status: StatusCode, message: &str) -> Response {
    json_response(status, json!({ "success": false, "error": message }))
}

async fn handle_manifest(State(plugin): State<Arc<Plugin>>) -> Response {
    Json(&plugin.manifest).into_response()
}

async fn handle_action_execute(
    State(plugin): State<Arc<Plugin>>,
    Path(action_id): Path<String>,
    body: axum::body::Bytes,
) -> Response {
    let action = match plugin.actions.get(&action_id) {
        Some(a) => a,
        None => return error_json(StatusCode::NOT_FOUND, &format!("Unknown action \"{}\"", action_id)),
    };

    let request: ExecuteActionRequest = match serde_json::from_slice(&body) {
        Ok(r) => r,
        Err(e) => return error_json(StatusCode::BAD_REQUEST, &format!("Invalid JSON: {}", e)),
    };

    let result = std::panic::AssertUnwindSafe((action.execute)(request));
    match result.await {
        resp => {
            let value = serde_json::to_value(&resp).unwrap_or_else(|_| {
                json!({ "success": false, "error": "Failed to serialize response" })
            });
            json_response(StatusCode::OK, value)
        }
    }
}

async fn handle_trigger_setup(
    State(plugin): State<Arc<Plugin>>,
    Path(trigger_id): Path<String>,
    body: axum::body::Bytes,
) -> Response {
    let trigger = match plugin.triggers.get(&trigger_id) {
        Some(t) => t,
        None => {
            return error_json(
                StatusCode::NOT_FOUND,
                &format!("Unknown trigger \"{}\"", trigger_id),
            )
        }
    };

    let handler = match &trigger.setup {
        Some(h) => h,
        None => {
            return error_json(
                StatusCode::NOT_FOUND,
                &format!("Trigger \"{}\" does not implement setup", trigger_id),
            )
        }
    };

    let request: SetupTriggerRequest = match serde_json::from_slice(&body) {
        Ok(r) => r,
        Err(e) => return error_json(StatusCode::BAD_REQUEST, &format!("Invalid JSON: {}", e)),
    };

    let resp = handler(request).await;
    let value = serde_json::to_value(&resp)
        .unwrap_or_else(|_| json!({ "success": false, "error": "Failed to serialize response" }));
    json_response(StatusCode::OK, value)
}

async fn handle_trigger_webhook(
    State(plugin): State<Arc<Plugin>>,
    Path(trigger_id): Path<String>,
    body: axum::body::Bytes,
) -> Response {
    let trigger = match plugin.triggers.get(&trigger_id) {
        Some(t) => t,
        None => {
            return error_json(
                StatusCode::NOT_FOUND,
                &format!("Unknown trigger \"{}\"", trigger_id),
            )
        }
    };

    let handler = match &trigger.webhook {
        Some(h) => h,
        None => {
            return error_json(
                StatusCode::NOT_FOUND,
                &format!("Trigger \"{}\" does not implement webhook", trigger_id),
            )
        }
    };

    let request: HandleTriggerWebhookRequest = match serde_json::from_slice(&body) {
        Ok(r) => r,
        Err(e) => return error_json(StatusCode::BAD_REQUEST, &format!("Invalid JSON: {}", e)),
    };

    let resp = handler(request).await;
    let value = serde_json::to_value(&resp)
        .unwrap_or_else(|_| json!({ "success": false, "error": "Failed to serialize response" }));
    json_response(StatusCode::OK, value)
}

async fn handle_trigger_cleanup(
    State(plugin): State<Arc<Plugin>>,
    Path(trigger_id): Path<String>,
    body: axum::body::Bytes,
) -> Response {
    let trigger = match plugin.triggers.get(&trigger_id) {
        Some(t) => t,
        None => {
            return error_json(
                StatusCode::NOT_FOUND,
                &format!("Unknown trigger \"{}\"", trigger_id),
            )
        }
    };

    let handler = match &trigger.cleanup {
        Some(h) => h,
        None => {
            return error_json(
                StatusCode::NOT_FOUND,
                &format!("Trigger \"{}\" does not implement cleanup", trigger_id),
            )
        }
    };

    let request: CleanupTriggerRequest = match serde_json::from_slice(&body) {
        Ok(r) => r,
        Err(e) => return error_json(StatusCode::BAD_REQUEST, &format!("Invalid JSON: {}", e)),
    };

    let resp = handler(request).await;
    let value = serde_json::to_value(&resp)
        .unwrap_or_else(|_| json!({ "success": false, "error": "Failed to serialize response" }));
    json_response(StatusCode::OK, value)
}
