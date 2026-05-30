use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Manifest types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterOption {
    pub label: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParameterManifest {
    pub id: String,
    pub label: String,
    #[serde(rename = "type")]
    pub param_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<ParameterOption>>,
}

impl Default for ParameterManifest {
    fn default() -> Self {
        Self {
            id: String::new(),
            label: String::new(),
            param_type: "string".into(),
            description: None,
            required: None,
            default: None,
            options: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionManifest {
    pub id: String,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub parameters: Vec<ParameterManifest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TriggerManifest {
    pub id: String,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub parameters: Vec<ParameterManifest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginManifest {
    pub id: String,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub actions: Vec<ActionManifest>,
    pub triggers: Vec<TriggerManifest>,
}

// ---------------------------------------------------------------------------
// Action request / response
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteActionRequest {
    pub parameters: HashMap<String, Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ExecuteActionResponse {
    Success {
        success: bool,
        data: HashMap<String, Value>,
    },
    Error {
        success: bool,
        error: String,
    },
}

impl ExecuteActionResponse {
    pub fn success(data: Value) -> Self {
        let map = match data {
            Value::Object(m) => m.into_iter().collect(),
            _ => {
                let mut map = HashMap::new();
                map.insert("result".into(), data);
                map
            }
        };
        ExecuteActionResponse::Success {
            success: true,
            data: map,
        }
    }

    pub fn error(error: impl Into<String>) -> Self {
        ExecuteActionResponse::Error {
            success: false,
            error: error.into(),
        }
    }
}

// ---------------------------------------------------------------------------
// Trigger setup
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookInfo {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupTriggerRequest {
    pub parameters: HashMap<String, Value>,
    pub webhook: WebhookInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SetupTriggerResponse {
    Success {
        success: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        metadata: Option<HashMap<String, Value>>,
    },
    Error {
        success: bool,
        error: String,
    },
}

impl SetupTriggerResponse {
    pub fn success(metadata: Option<HashMap<String, Value>>) -> Self {
        SetupTriggerResponse::Success {
            success: true,
            metadata,
        }
    }

    pub fn error(error: impl Into<String>) -> Self {
        SetupTriggerResponse::Error {
            success: false,
            error: error.into(),
        }
    }
}

// ---------------------------------------------------------------------------
// Trigger webhook handling
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebhookHttpResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ForwardedRequest {
    pub method: String,
    pub headers: HashMap<String, Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<HashMap<String, Vec<String>>>,
    pub raw_body_base64: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandleTriggerWebhookRequest {
    pub parameters: HashMap<String, Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, Value>>,
    pub request: ForwardedRequest,
}

/// The webhook response is a union type. Custom Serialize produces flat JSON
/// (no external tag) matching the wire protocol.
#[derive(Debug, Clone)]
pub enum HandleTriggerWebhookResponse {
    Emit {
        event_type: String,
        payload: Value,
        response: Option<WebhookHttpResponse>,
    },
    Skip {
        reason: Option<String>,
        response: Option<WebhookHttpResponse>,
    },
    Error {
        error: String,
        status: Option<u16>,
    },
}

impl HandleTriggerWebhookResponse {
    pub fn emit(
        event_type: impl Into<String>,
        payload: Value,
        response: Option<WebhookHttpResponse>,
    ) -> Self {
        HandleTriggerWebhookResponse::Emit {
            event_type: event_type.into(),
            payload,
            response,
        }
    }

    pub fn skip(reason: Option<String>, response: Option<WebhookHttpResponse>) -> Self {
        HandleTriggerWebhookResponse::Skip { reason, response }
    }

    pub fn error(error: impl Into<String>, status: Option<u16>) -> Self {
        HandleTriggerWebhookResponse::Error {
            error: error.into(),
            status,
        }
    }
}

impl Serialize for HandleTriggerWebhookResponse {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;

        match self {
            HandleTriggerWebhookResponse::Emit {
                event_type,
                payload,
                response,
            } => {
                let mut count = 4;
                if response.is_some() {
                    count += 1;
                }
                let mut map = serializer.serialize_map(Some(count))?;
                map.serialize_entry("success", &true)?;
                map.serialize_entry("emit", &true)?;
                map.serialize_entry("eventType", event_type)?;
                map.serialize_entry("payload", payload)?;
                if let Some(resp) = response {
                    map.serialize_entry("response", resp)?;
                }
                map.end()
            }
            HandleTriggerWebhookResponse::Skip { reason, response } => {
                let mut count = 2;
                if reason.is_some() {
                    count += 1;
                }
                if response.is_some() {
                    count += 1;
                }
                let mut map = serializer.serialize_map(Some(count))?;
                map.serialize_entry("success", &true)?;
                map.serialize_entry("emit", &false)?;
                if let Some(r) = reason {
                    map.serialize_entry("reason", r)?;
                }
                if let Some(resp) = response {
                    map.serialize_entry("response", resp)?;
                }
                map.end()
            }
            HandleTriggerWebhookResponse::Error { error, status } => {
                let mut count = 2;
                if status.is_some() {
                    count += 1;
                }
                let mut map = serializer.serialize_map(Some(count))?;
                map.serialize_entry("success", &false)?;
                map.serialize_entry("error", error)?;
                if let Some(s) = status {
                    map.serialize_entry("status", s)?;
                }
                map.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for HandleTriggerWebhookResponse {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let v = Value::deserialize(deserializer)?;
        let obj = v
            .as_object()
            .ok_or_else(|| serde::de::Error::custom("expected object"))?;

        let success = obj
            .get("success")
            .and_then(Value::as_bool)
            .ok_or_else(|| serde::de::Error::custom("missing success field"))?;

        if !success {
            let error = obj
                .get("error")
                .and_then(Value::as_str)
                .unwrap_or("Unknown error")
                .to_string();
            let status = obj.get("status").and_then(Value::as_u64).map(|s| s as u16);
            return Ok(HandleTriggerWebhookResponse::Error { error, status });
        }

        let emit = obj.get("emit").and_then(Value::as_bool).unwrap_or(false);

        if emit {
            let event_type = obj
                .get("eventType")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            let payload = obj.get("payload").cloned().unwrap_or(Value::Null);
            let response: Option<WebhookHttpResponse> = obj
                .get("response")
                .map(|v| serde_json::from_value(v.clone()))
                .transpose()
                .map_err(serde::de::Error::custom)?;
            Ok(HandleTriggerWebhookResponse::Emit {
                event_type,
                payload,
                response,
            })
        } else {
            let reason = obj.get("reason").and_then(Value::as_str).map(String::from);
            let response: Option<WebhookHttpResponse> = obj
                .get("response")
                .map(|v| serde_json::from_value(v.clone()))
                .transpose()
                .map_err(serde::de::Error::custom)?;
            Ok(HandleTriggerWebhookResponse::Skip { reason, response })
        }
    }
}

// ---------------------------------------------------------------------------
// Trigger cleanup
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupTriggerRequest {
    pub parameters: HashMap<String, Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CleanupTriggerResponse {
    Success { success: bool },
    Error { success: bool, error: String },
}

impl CleanupTriggerResponse {
    pub fn success() -> Self {
        CleanupTriggerResponse::Success { success: true }
    }

    pub fn error(error: impl Into<String>) -> Self {
        CleanupTriggerResponse::Error {
            success: false,
            error: error.into(),
        }
    }
}

// ---------------------------------------------------------------------------
// Direct events
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectPluginEvent {
    pub event_type: String,
    pub payload: Value,
}
