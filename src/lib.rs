//! table-manners, the shared contract of the waffle tools (CONTRACT.md): serde types for its
//! shared objects and the JSON Schemas they match. The core's types are at the root; the images
//! profile's are in [`images`]. No logic: a tool computes digests, verifies references and writes
//! results itself, and validates against [`SCHEMAS`] in its own tests.
//!
//! Section numbers in these docs are CONTRACT.md's.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

pub mod images;

/// The contract version this crate describes; every result and description carries it.
pub const CONTRACT_VERSION: &str = "0.1.0";

/// Every schema as `($id, text)`. References between them use the `$id`s, so a validator needs a
/// retriever that serves this list and nothing else.
pub const SCHEMAS: &[(&str, &str)] = &[
    (
        "urn:table-manners:images:image-reference",
        include_str!("../schemas/images/image-reference.json"),
    ),
    (
        "urn:table-manners:images:location",
        include_str!("../schemas/images/location.json"),
    ),
    (
        "urn:table-manners:images:image-in",
        include_str!("../schemas/images/image-in.json"),
    ),
    (
        "urn:table-manners:images:image-out",
        include_str!("../schemas/images/image-out.json"),
    ),
    (
        "urn:table-manners:core:result",
        include_str!("../schemas/core/result.json"),
    ),
    (
        "urn:table-manners:core:description",
        include_str!("../schemas/core/description.json"),
    ),
    (
        "urn:table-manners:core:runs",
        include_str!("../schemas/core/runs.json"),
    ),
];

/// A profile: a named set of additions to the core's objects (1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Profile {
    /// Tools that take and return images (part 2, [`images`]).
    Images,
}

/// A run's result (2): the MCP result's `structuredContent`, the run resource and `run.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunResult {
    /// The contract version the tool conforms to.
    pub contract: String,
    /// The tool.
    pub tool: Tool,
    /// The action's name.
    pub action: String,
    /// Where the run lives.
    pub run: Run,
    /// How it ended (2.1).
    pub status: Status,
    /// Present exactly when `status` is `error` or `invalid` (2.2).
    pub error: Option<ErrorInfo>,
    /// RFC 3339, UTC.
    pub started: String,
    /// RFC 3339, UTC.
    pub finished: String,
    /// Wall time.
    pub duration_ms: u64,
    /// `sha256:<hex>` over the resolved inputs and parameters, by the tool's recipe.
    pub input_identity: String,
    /// What the run cost.
    pub spend: Spend,
    /// Whether the paid step came from the cache.
    pub cache: Cache,
    /// Images profile (11.1): the images taken in and returned.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub images: Option<images::Images>,
    /// Images profile (11.1): the locations taken in and produced.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub locations: Option<images::Locations>,
    /// Caveats on a run, in sentences.
    pub notes: Vec<String>,
    /// The tool's own.
    pub body: serde_json::Map<String, serde_json::Value>,
}

/// A tool's name and version.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tool {
    /// The tool's name, also its resource scheme.
    pub name: String,
    /// The tool's version.
    pub version: String,
}

/// Where a run lives.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Run {
    /// Unique per tool.
    pub id: String,
    /// `<tool>://run/<id>/run.json`.
    pub resource: String,
    /// The run directory on a local transport, else none.
    pub dir: Option<String>,
}

/// How a run ended (2.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    /// The action ran to its end.
    Completed,
    /// Stopped by the caller or a signal.
    Cancelled,
    /// An operational failure.
    Error,
    /// The request was wrong.
    Invalid,
}

/// What went wrong (2.2).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ErrorInfo {
    /// The class.
    pub code: ErrorCode,
    /// The tool's judgement for this occurrence.
    pub retryable: bool,
    /// What happened, in a sentence.
    pub reason: String,
}

/// The error classes (2.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    /// An unknown action, a missing or malformed argument.
    Usage,
    /// An image or location the tool cannot take.
    Input,
    /// Reading or writing the tool's own files failed.
    Io,
    /// A remote model or service failed.
    Provider,
    /// The per-call cap or the session budget would be exceeded.
    Budget,
    /// A bounded wait ran out.
    Timeout,
    /// A helper process died, hung or answered out of contract.
    ToolProcess,
    /// A bug in the tool.
    Internal,
}

/// What a run cost.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Spend {
    /// US dollars, including a cancelled or failed run's spend to date; 0 for a cache hit.
    pub usd: f64,
    /// How the figure was obtained.
    pub basis: SpendBasis,
}

/// How a spend figure was obtained.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpendBasis {
    /// Metered by the provider.
    Measured,
    /// From a configured price.
    Estimated,
}

/// Whether the paid step came from the cache.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Cache {
    /// Answered from the cache.
    Hit,
    /// Computed and stored.
    Miss,
    /// The cache was not consulted.
    Off,
}

/// The description document (3).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Description {
    /// The contract version the tool conforms to.
    pub contract: String,
    /// The profiles the tool implements; empty for a core-only tool.
    pub profiles: Vec<Profile>,
    /// The tool.
    pub tool: DescribedTool,
    /// One per MCP tool, `describe` excepted.
    pub actions: Vec<Action>,
    /// Images profile (11.2): every word the tool emits in a location.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vocabulary: Option<images::Vocabulary>,
    /// Images profile (11.2): size limits.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limits: Option<images::Limits>,
    /// The longest gap between progress notifications on a live run, at most 10 seconds.
    pub progress_interval_s: u32,
    /// Resource URI templates.
    pub resources: Vec<String>,
    /// `x-<tool>` keys: the tool's own.
    #[serde(flatten)]
    pub extensions: BTreeMap<String, serde_json::Value>,
}

/// A tool, described.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DescribedTool {
    /// The tool's name.
    pub name: String,
    /// The tool's version.
    pub version: String,
    /// What it is for, in a sentence.
    pub purpose: String,
}

/// One action (3).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Action {
    /// The MCP tool name.
    pub name: String,
    /// A short title.
    pub title: String,
    /// What it does, in a sentence.
    pub purpose: String,
    /// A JSON Schema for the arguments other than those a profile adds.
    pub params_schema: serde_json::Value,
    /// Images profile (11.2): image slots.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<images::ImageSlot>>,
    /// Images profile (11.2): location slots.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub locations: Option<Vec<images::LocationSlot>>,
    /// Images profile (11.2): what it returns.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outputs: Option<images::Outputs>,
    /// Whether and how it spends.
    pub cost: Cost,
    /// How long it takes.
    pub duration: Duration,
    /// Commands that render a run.
    pub renders: Vec<Render>,
}

/// Whether and how an action spends.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Cost {
    /// Whether a call may cost money.
    pub may_spend: bool,
    /// How its spend is reported; required when it may spend.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub basis: Option<SpendBasis>,
    /// Environment variables it needs, by name only.
    pub needs_env: Vec<String>,
}

/// How long an action takes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Duration {
    /// Typical seconds.
    pub typical_s: f64,
    /// The most seconds it may take.
    pub max_s: f64,
}

/// A command that renders a run; `{run.dir}` is substituted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Render {
    /// Its name.
    pub name: String,
    /// The argument vector.
    pub command: Vec<String>,
}

/// The `<tool>://runs` resource (4.3).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunsList {
    /// The contract version the tool conforms to.
    pub contract: String,
    /// The tool.
    pub tool: Tool,
    /// Recent runs, newest first.
    pub runs: Vec<RunSummary>,
}

/// One run in the list.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunSummary {
    /// The run's id.
    pub id: String,
    /// The action's name.
    pub action: String,
    /// Where it is.
    pub status: RunState,
    /// RFC 3339, UTC.
    pub started: String,
    /// `<tool>://run/<id>/run.json`.
    pub resource: String,
}

/// A listed run's state: a result's status, or still running.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunState {
    /// Not finished; `run.json` does not exist yet.
    Running,
    /// See [`Status::Completed`].
    Completed,
    /// See [`Status::Cancelled`].
    Cancelled,
    /// See [`Status::Error`].
    Error,
    /// See [`Status::Invalid`].
    Invalid,
}
