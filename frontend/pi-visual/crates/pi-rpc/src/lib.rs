//! A deliberately small and UI-independent boundary for the official Pi RPC JSONL protocol.
//!
//! Validates inbound records, constructs read-only requests and correlates replies.
//! Includes async transport, deterministic workflow reduction and diagnostic probes.

pub mod client;
pub mod extension_ui;
pub mod framing;
pub mod reducer;
mod terminal_text;
pub mod transport;

pub mod protocol;
pub use protocol::*;
