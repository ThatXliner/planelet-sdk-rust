pub mod client;
pub mod plugin;
pub mod types;
pub mod utils;

// Re-export primary types and functions for convenience.
pub use client::{SuperPlaneClient, SuperPlaneClientError, SuperPlaneClientOptions};
pub use plugin::{
    ActionDefinition, ActionHandler, Plugin, PluginBuilder, TriggerCleanupHandler,
    TriggerDefinition, TriggerSetupHandler, TriggerWebhookHandler,
};
pub use types::*;
pub use utils::{decode_raw_body, decode_raw_body_text, first_header};
