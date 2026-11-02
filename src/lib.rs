//! table-manners, the shared contract of the waffle tools (CONTRACT.md): serde types for its
//! shared objects and the JSON Schemas they match. The core's types are at the root; the images
//! profile's are in [`images`] and the operations profile's in [`operations`]. No logic: a tool computes digests, verifies references and writes
//! results itself, and validates against [`SCHEMAS`] in its own tests.
//!
//! Section numbers in these docs are CONTRACT.md's.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

pub mod images;
pub mod operations;

/// The contract version this crate describes; every result and description carries it.
pub const CONTRACT_VERSION: &str = "0.4.0";

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
    (
        "urn:table-manners:core:run-ref",
        include_str!("../schemas/core/run-ref.json"),
    ),
    (
        "urn:table-manners:core:session",
        include_str!("../schemas/core/session.json"),
    ),
    (
        "urn:table-manners:core:refusal",
        include_str!("../schemas/core/refusal.json"),
    ),
    (
        "urn:table-manners:core:estimate",
        include_str!("../schemas/core/estimate.json"),
    ),
    (
        "urn:table-manners:core:invocation-bundle",
        include_str!("../schemas/core/invocation-bundle.json"),
    ),
    (
        "urn:table-manners:operations:artifact-type",
        include_str!("../schemas/operations/artifact-type.json"),
    ),
    (
        "urn:table-manners:operations:artifact-ref",
        include_str!("../schemas/operations/artifact-ref.json"),
    ),
    (
        "urn:table-manners:operations:artifact",
        include_str!("../schemas/operations/artifact.json"),
    ),
    (
        "urn:table-manners:operations:port",
        include_str!("../schemas/operations/port.json"),
    ),
    (
        "urn:table-manners:operations:binding",
        include_str!("../schemas/operations/binding.json"),
    ),
    (
        "urn:table-manners:operations:export-request",
        include_str!("../schemas/operations/export-request.json"),
    ),
    (
        "urn:table-manners:operations:export-response",
        include_str!("../schemas/operations/export-response.json"),
    ),
    (
        "urn:table-manners:operations:import-request",
        include_str!("../schemas/operations/import-request.json"),
    ),
    (
        "urn:table-manners:operations:import-response",
        include_str!("../schemas/operations/import-response.json"),
    ),
];

/// A profile: a named set of additions to the core's objects (1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Profile {
    /// Tools that take and return images (part 2, [`images`]).
    Images,
    /// Single steps a host runs alone on typed artifacts ([`operations`]); requires `images`.
    Operations,
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
    /// What the call resolved to (4.8); none only on an `invalid` run whose arguments did not
    /// resolve.
    pub invocation: Option<Invocation>,
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
    /// Operations profile (4.7): the artifacts taken through ports and made.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artifacts: Option<operations::Artifacts>,
    /// Caveats on a run, in sentences.
    pub notes: Vec<String>,
    /// The tool's own.
    pub body: serde_json::Map<String, serde_json::Value>,
}

/// What a call resolved to (4.8), so it can be repeated from another directory, by another client
/// or on another machine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Invocation {
    /// The action's name.
    pub action: String,
    /// The arguments as resolved: defaults applied, each image as `{"expect": <reference>}` with
    /// no locator, each artifact as its reference, delivery-only arguments left out.
    pub arguments: serde_json::Map<String, serde_json::Value>,
    /// What the call used where its arguments left the choice to the tool (4.5).
    pub effective: Effective,
    /// The name of each readiness requirement the call used; never a value.
    pub requirements: Vec<String>,
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

/// A run's identity (4.3): progress `_meta` under `table-manners/run`, and inside a reuse
/// [`Refusal`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunRef {
    /// Unique per tool.
    pub id: String,
    /// `<tool>://run/<id>/run.json`.
    pub resource: String,
    /// The serving session (4.6); none for a run made outside one (a tool's CLI).
    pub session_id: Option<String>,
    /// The caller's correlation id (4.4); none when the request sent none.
    pub client_run_id: Option<String>,
}

/// Where a run lives: its [`RunRef`] fields and its directory.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Run {
    /// Unique per tool.
    pub id: String,
    /// `<tool>://run/<id>/run.json`.
    pub resource: String,
    /// The serving session (4.6); none for a run made outside one (a tool's CLI).
    pub session_id: Option<String>,
    /// The caller's correlation id (4.4); none when the request sent none.
    pub client_run_id: Option<String>,
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
    /// The call named an estimate that no longer matches it, or has expired (4.5).
    StaleEstimate,
    /// Reading or writing the tool's own files failed.
    Io,
    /// A remote model or service failed.
    Provider,
    /// A provider rejected the credential (401 or 403) during the run; never retryable. A
    /// credential missing before the run is the `not_ready` refusal.
    Credential,
    /// The call's worst case is over its effective per-call cap (4.5).
    Cap,
    /// The session budget cannot reserve the call's worst case (4.5).
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
    /// The part of `usd` no provider answer settled (4.5): an upper bound, not a charge; never
    /// more than `usd`.
    pub unresolved_usd: f64,
    /// How the figure was obtained.
    pub basis: SpendBasis,
}

/// How a spend figure was obtained.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpendBasis {
    /// Reported by the provider, as a charge or as usage priced per unit.
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
    /// One per MCP tool, the reserved `describe`, `estimate`, `export` and `import` excepted.
    pub actions: Vec<Action>,
    /// Word lists (3): the core's `group` and the images profile's (11.2).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vocabulary: Option<Vocabulary>,
    /// Operations profile (3.3): every type the tool's ports name, except `image`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artifact_types: Option<Vec<operations::ArtifactType>>,
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
    /// What kind of action it is; a host shows actions by kind.
    pub kind: ActionKind,
    /// A word of the tool's own that sorts actions within a kind, listed in `vocabulary.group`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
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
    /// Operations profile (4.7): typed inputs and outputs; on every operation and no other kind.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ports: Option<operations::Ports>,
    /// Whether and how it spends.
    pub cost: Cost,
    /// Whether it can run here, now (3.1).
    pub readiness: Readiness,
    /// How long it takes.
    pub duration: Duration,
    /// Commands that render a run.
    pub renders: Vec<Render>,
}

/// What kind of action it is (3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionKind {
    /// A whole job a person or agent asks for.
    Task,
    /// One step on typed artifacts, run alone (4.7); only in a tool that lists `operations`.
    Operation,
    /// Neither.
    Utility,
}

/// The description's word lists (3). The core owns `group`; the images profile adds the words a
/// tool emits in a location (9.2, 11.2), all four required with it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Vocabulary {
    /// Every group an action names; required whenever any action has one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group: Option<Vec<String>>,
    /// Images profile: `disposition` words.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disposition: Option<Vec<String>>,
    /// Images profile: `confidence` words.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence: Option<Vec<String>>,
    /// Images profile: `evidence` words.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence: Option<Vec<String>>,
    /// Images profile: `score_kind` words.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub score_kind: Option<Vec<String>>,
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
}

/// Whether an action can run here, now, and what stands in the way (3.1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Readiness {
    /// The action's state.
    pub state: ReadinessState,
    /// What it needs, met or not.
    pub requirements: Vec<Requirement>,
}

/// An action's readiness (3.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReadinessState {
    /// No requirement is missing or invalid.
    Ready,
    /// A requirement is missing or invalid, and a person can fix it.
    NeedsSetup,
    /// This build or platform cannot run the action.
    Unavailable,
}

/// One thing an action needs (3.1). Never a credential value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Requirement {
    /// What kind of thing.
    pub kind: RequirementKind,
    /// A variable, file or entry name.
    pub name: String,
    /// Whether it is there and accepted.
    pub state: RequirementState,
    /// A sentence a person can act on.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub setup: Option<String>,
}

/// The kinds of requirement (3.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequirementKind {
    /// The tool's configuration, or an entry in it.
    Config,
    /// A credential, by variable name.
    Credential,
    /// A variable the tool reads that is not a secret.
    Env,
    /// A model or endpoint the action calls, as configured.
    Model,
    /// A local file it needs, such as model weights.
    File,
    /// A helper program or library.
    Dependency,
}

/// A requirement's state (3.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequirementState {
    /// Not there.
    Missing,
    /// There, and not checked.
    Present,
    /// A provider accepted it in this session.
    Valid,
    /// There and rejected, or it does not parse.
    Invalid,
    /// The tool cannot tell.
    Unknown,
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

/// A command that shows a run to a person (3.2); `{run.dir}` and, for a server,
/// `{frame_ancestor}` are substituted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Render {
    /// Its name.
    pub name: String,
    /// What it does when run.
    pub kind: RenderKind,
    /// Server renders: the longest wait for the URL line, at most 60 seconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ready_timeout_s: Option<u32>,
    /// The argument vector.
    pub command: Vec<String>,
}

/// What a render does (3.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RenderKind {
    /// Writes a file, prints its absolute path, exits.
    Document,
    /// Prints its loopback URL once listening; serves until stdin closes or SIGTERM.
    Server,
}

/// A page of the `<tool>://runs{?cursor}` resource (4.3).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunsList {
    /// The contract version the tool conforms to.
    pub contract: String,
    /// The tool.
    pub tool: Tool,
    /// Runs, newest first.
    pub runs: Vec<RunSummary>,
    /// Opaque, URI-unreserved characters only; the next page is `<tool>://runs?cursor=<next_cursor>`.
    /// None on the last page.
    pub next_cursor: Option<String>,
}

/// One run in the list.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunSummary {
    /// The run's id.
    pub id: String,
    /// `<tool>://run/<id>/run.json`.
    pub resource: String,
    /// The serving session (4.6); none for a run made outside one.
    pub session_id: Option<String>,
    /// The caller's correlation id (4.4); none when the request sent none.
    pub client_run_id: Option<String>,
    /// The action's name.
    pub action: String,
    /// Where it is.
    pub status: RunState,
    /// RFC 3339, UTC.
    pub started: String,
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

/// The `<tool>://session` resource (4.6): one atomic observation of the serving process's ledger.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    /// The contract version the tool conforms to.
    pub contract: String,
    /// The tool.
    pub tool: Tool,
    /// A random UUID (v4), generated when the server starts.
    pub id: String,
    /// RFC 3339, UTC.
    pub started: String,
    /// The session budget; none when uncapped.
    pub budget_usd: Option<f64>,
    /// Settled spend, `unresolved_usd` included.
    pub spent_usd: f64,
    /// The part of `spent_usd` no provider answer settled.
    pub unresolved_usd: f64,
    /// Worst cases held by runs still going.
    pub reserved_usd: f64,
    /// `max(0, budget_usd - spent_usd - reserved_usd)`; none exactly when `budget_usd` is.
    pub remaining_usd: Option<f64>,
    /// The directories the session reads caller paths from (5.2): absolute, symlinks resolved,
    /// narrowed by the configuration; empty when every path is refused.
    pub allowed_dirs: Vec<String>,
}

/// The `data` of a JSON-RPC `-32602` error the contract names (3.1, 4.4, 4.5, 4.7, 5.1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Refusal {
    /// Why.
    pub reason: RefusalReason,
    /// `client_run_id_reused`: the earlier request's run, none while it has none; every other
    /// reason: none.
    pub run: Option<RunRef>,
    /// `not_ready`: the requirements the call needs that are missing or invalid.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requirements: Option<Vec<Requirement>>,
    /// `invalid_arguments`: what was wrong (`usage` or `input`, not retryable).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorInfo>,
    /// `binding`: one entry per reference that failed, in port order (4.7).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bindings: Option<Vec<operations::Binding>>,
}

/// The refusals the contract names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RefusalReason {
    /// The request's `client_run_id` was already used in this session.
    ClientRunIdReused,
    /// The call needs a requirement that is missing or invalid (3.1).
    NotReady,
    /// A control's request was wrong: an estimate (4.5), an export or an import (5.1).
    InvalidArguments,
    /// An operation's artifacts did not bind, or a read named an identity not held (4.7).
    Binding,
}

/// What the reserved `estimate` tool returns (4.5): the worst case of one exact call, bound to
/// its inputs, configuration, tool version and session. Never a run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Estimate {
    /// The contract version the tool conforms to.
    pub contract: String,
    /// The tool.
    pub tool: Tool,
    /// The paid action priced.
    pub action: String,
    /// `sha256:<hex>`; sent back as `_meta["table-manners/estimate"]` on the paid call.
    pub binding: String,
    /// The serving session the binding belongs to; none for a command-line estimate.
    pub session_id: Option<String>,
    /// What the call would use where its arguments leave the choice to the tool.
    pub effective: Effective,
    /// What the limits check would reserve; none only when the call cannot be priced.
    pub worst_case_usd: Option<f64>,
    /// The effective per-call cap; none when uncapped, never a zero allowance.
    pub cap_usd: Option<f64>,
    /// Whether the limits check would pass now.
    pub allowed: bool,
    /// Why not, when `allowed` is false.
    pub refused: Option<EstimateRefusal>,
    /// The action's readiness (3.1).
    pub readiness: Readiness,
    /// One atomic observation of the session (4.6); none for a command-line estimate.
    pub session: Option<Session>,
    /// When the tool stops honouring the binding.
    pub expires: String,
}

/// What a call uses where its arguments leave the choice to the tool (4.5); also an
/// [`Invocation`]'s (4.8).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Effective {
    /// The model or endpoint; none when the action calls none.
    pub model: Option<String>,
    /// `sha256:<hex>` of the configuration in force; none when there is no configuration.
    pub config_identity: Option<String>,
    /// `x-<tool>` keys: the tool's own.
    #[serde(flatten)]
    pub extensions: BTreeMap<String, serde_json::Value>,
}

/// Why an estimated call would not pass the limits check (4.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EstimateRefusal {
    /// Its worst case is over the effective per-call cap.
    Cap,
    /// The session cannot hold its worst case.
    Budget,
    /// The action is not ready.
    Readiness,
}

/// `invocation.json`, the manifest of a portable invocation bundle (4.9). A host builds it; no tool
/// does.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InvocationBundle {
    /// 1.
    pub bundle_version: u32,
    /// The calls in order; a single call is one step.
    pub steps: Vec<BundleStep>,
}

/// One call in an invocation bundle (4.9).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BundleStep {
    /// The contract version the tool conformed to.
    pub contract: String,
    /// The tool.
    pub tool: Tool,
    /// What the call resolved to (4.8).
    pub invocation: Invocation,
    /// The original `run.json`.
    pub result: Box<RunResult>,
    /// Each image the step took, with its member.
    pub inputs: Vec<BundleInput>,
    /// Each native input with its export file.
    pub artifacts: Vec<BundleArtifact>,
    /// Each input left out on purpose.
    pub reference_only: Vec<ReferenceOnly>,
    /// Each native input that could not be exported.
    pub unportable: Vec<Unportable>,
}

/// An image a bundled step took: by `slot` (images profile) or `port` (operations), exactly one.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BundleInput {
    /// The image slot it came through.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slot: Option<String>,
    /// The port it came through.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub port: Option<String>,
    /// Its reference.
    pub image: images::ImageReference,
    /// `inputs/<file_sha256>.<ext>`: the caller's file, byte for byte.
    pub member: String,
}

/// A native input of a bundled step, with its export file (5.1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BundleArtifact {
    /// The port it came through.
    pub port: String,
    /// Its reference.
    pub artifact: operations::ArtifactRef,
    /// `artifacts/<identity>.<ext>`, the identity's `:` written `-`.
    pub member: String,
    /// SHA-256 of the export file, lowercase hex.
    pub sha256: String,
}

/// An input left out of a bundle on purpose: by `slot` or `port`, exactly one.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferenceOnly {
    /// The image slot.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slot: Option<String>,
    /// The port.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub port: Option<String>,
    /// Why, in a sentence.
    pub why: String,
}

/// A native input a bundle could not carry; the bundle is workspace-dependent.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Unportable {
    /// The port it came through.
    pub port: String,
    /// Its reference.
    pub artifact: operations::ArtifactRef,
    /// Why: its type is `portable: none`, or its export failed.
    pub reason: String,
}
