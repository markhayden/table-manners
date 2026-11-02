//! The operations profile (CONTRACT.md 3.3, 3.4, 4.7, 5.1): artifact types and references, ports,
//! the entries of the `binding` refusal, artifacts in the result, the artifact resource, and the
//! reserved `export` and `import` tools.

use serde::{Deserialize, Serialize};

use crate::Tool;
use crate::images::{ImageOut, ImageReference};

/// One entry of the description's `artifact_types` (3.3). `image` is the contract's and is never
/// listed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArtifactType {
    /// `location`, or the tool's own `<tool.name>:<Name>`.
    #[serde(rename = "type")]
    pub artifact_type: String,
    /// The versions of the type's record the tool reads; it writes the newest.
    pub schema_versions: Vec<u32>,
    /// What it is, in a sentence.
    pub purpose: String,
    /// How an artifact of the type leaves its workspace.
    pub portable: Portable,
    /// Whether the tool draws an artifact of the type as an image for people (4.7).
    pub preview: bool,
}

/// How an artifact leaves the workspace it lives in (3.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Portable {
    /// A file the tool writes and reads whole (5.1).
    Bundle,
    /// A JSON document (5.1).
    Json,
    /// It cannot leave yet; a host labels it workspace-dependent.
    None,
}

/// How an operation names an artifact (3.4). Never a workspace id.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArtifactRef {
    /// One of the tool's artifact types, or `image`.
    #[serde(rename = "type")]
    pub artifact_type: String,
    /// One of the type's schema versions; 1 for `image` and `location`.
    pub schema_version: u32,
    /// The tool that made it.
    pub tool: String,
    /// `sha256:<hex>` by the tool's recipe; for an image, `sha256:` and its `file_sha256`.
    pub identity: String,
    /// The tool's own, opaque identity of the pixel grid it is drawn in; none for a frameless type.
    pub frame: Option<String>,
    /// The root image it was made on, when the tool knows it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub on: Option<ImageReference>,
    /// `<tool>://artifact/{identity}`.
    pub resource: String,
}

/// A typed input or output of an operation (4.7).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Port {
    /// Lowercase with underscores; unique within `in` and within `out`.
    pub name: String,
    /// `image`, `location` or one of the description's artifact types.
    #[serde(rename = "type")]
    pub artifact_type: String,
    /// Whether the call must fill it (in), or the run always makes it (out).
    pub required: bool,
    /// Whether it takes or makes a non-empty list; false when absent.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub many: bool,
    /// What it is for, in a sentence.
    pub purpose: String,
}

/// An operation's ports (4.7).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ports {
    /// Inputs; each is also a property of the operation's MCP input schema.
    #[serde(rename = "in")]
    pub inputs: Vec<Port>,
    /// Outputs.
    pub out: Vec<Port>,
}

/// One reference that did not bind, in a `binding` refusal (4.7).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Binding {
    /// The in-port, `artifact` for export; none for a read of the artifact resource.
    pub port: Option<String>,
    /// The item's 0-based position within a `many` port; only for such items.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index: Option<u32>,
    /// What is wrong.
    pub problem: BindingProblem,
    /// What the port takes, for people.
    pub expected: String,
    /// What it was given, for people.
    pub got: String,
    /// A sentence a person can act on; required for `unresolved`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remedy: Option<String>,
}

/// Why a reference did not bind (4.7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BindingProblem {
    /// A type the port does not take, or another tool's type.
    WrongType,
    /// A schema version the tool does not read.
    WrongVersion,
    /// A frame the operation cannot use with its other inputs.
    WrongFrame,
    /// An identity the tool's store does not hold.
    Unresolved,
}

/// The artifacts a run took and made (4.7): required in every result of an operations tool.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Artifacts {
    /// Taken through a port.
    #[serde(rename = "in")]
    pub taken: Vec<ArtifactEntry>,
    /// Made, a task's native outputs included.
    pub out: Vec<ArtifactEntry>,
}

/// An artifact a run took or made (4.7).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArtifactEntry {
    /// The port, or the task's output, it went through.
    pub port: String,
    /// Its reference.
    pub artifact: ArtifactRef,
    /// An image out with role `preview`: required on an out entry whose type declares one, none
    /// otherwise; optional on an in entry.
    pub preview: Option<ImageOut>,
}

/// What `<tool>://artifact/{identity}` returns (4.7).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArtifactResource {
    /// The contract version the tool conforms to.
    pub contract: String,
    /// The tool.
    pub tool: Tool,
    /// Its reference.
    pub artifact: ArtifactRef,
    /// An image out with role `preview`; none when the type has none.
    pub preview: Option<ImageOut>,
    /// The tool's own record; a host never reads it.
    pub record: serde_json::Map<String, serde_json::Value>,
}

/// The input of the reserved `export` tool (5.1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExportRequest {
    /// The artifact to write in its portable form.
    pub artifact: ArtifactRef,
}

/// What the reserved `export` tool returns (5.1). Never a run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExportResponse {
    /// The contract version the tool conforms to.
    pub contract: String,
    /// The tool.
    pub tool: Tool,
    /// The artifact exported.
    pub artifact: ArtifactRef,
    /// Where its portable form was written.
    pub file: ExportedFile,
}

/// A file `export` wrote in the tool's own data directory (5.1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExportedFile {
    /// The absolute path the tool chose, outside any run directory.
    pub path: String,
    /// SHA-256 of its bytes, lowercase hex.
    pub sha256: String,
    /// Its size.
    pub bytes: u64,
    /// The bundle's media type, or `application/json`.
    pub media_type: String,
}

/// The input of the reserved `import` tool (5.1): a file by path or its bytes inline.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ImportRequest {
    /// By path, under the path rule (5.2).
    File(ImportFile),
    /// Inline, up to `limits.inline_bytes`.
    Inline(ImportInline),
}

/// An import by path.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImportFile {
    /// The file.
    pub file: FilePath,
    /// Checked against the bytes, when given.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
}

/// A file named by path.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FilePath {
    /// An absolute path inside the session's allowed directories (5.2).
    pub path: String,
}

/// An import of inline bytes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImportInline {
    /// The bytes.
    pub inline: InlineFile,
    /// Checked against the bytes, when given.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
}

/// Base64 file bytes with their media type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InlineFile {
    /// `application/json`, `application/x-tar`, ….
    pub media_type: String,
    /// Standard base64.
    pub data: String,
}

/// What the reserved `import` tool returns (5.1). Never a run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImportResponse {
    /// The contract version the tool conforms to.
    pub contract: String,
    /// The tool.
    pub tool: Tool,
    /// The artifact now in the tool's store.
    pub artifact: ArtifactRef,
    /// True when the store already held it and nothing new was recorded.
    pub already_present: bool,
}
