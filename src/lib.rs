//! table-manners, the shared contract of the waffle tools (CONTRACT.md): serde types for its shared objects and the JSON
//! Schemas they match. No logic: a tool computes digests, verifies references and writes results
//! itself, and validates against [`SCHEMAS`] in its own tests.
//!
//! Section numbers in these docs are CONTRACT.md's.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// The contract version this crate describes; every result and description carries it.
pub const CONTRACT_VERSION: &str = "0.1.0";

/// Every schema as `($id, text)`. References between them use the `$id`s, so a validator needs a
/// retriever that serves this list and nothing else.
pub const SCHEMAS: &[(&str, &str)] = &[
    (
        "urn:table-manners:image-reference",
        include_str!("../schemas/image-reference.json"),
    ),
    (
        "urn:table-manners:location",
        include_str!("../schemas/location.json"),
    ),
    (
        "urn:table-manners:image-in",
        include_str!("../schemas/image-in.json"),
    ),
    (
        "urn:table-manners:image-out",
        include_str!("../schemas/image-out.json"),
    ),
    (
        "urn:table-manners:result",
        include_str!("../schemas/result.json"),
    ),
    (
        "urn:table-manners:description",
        include_str!("../schemas/description.json"),
    ),
    (
        "urn:table-manners:runs",
        include_str!("../schemas/runs.json"),
    ),
];

/// How a tool names an image to another tool (2).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImageReference {
    /// SHA-256 of the file's bytes, lowercase hex; the enforced check (2.1).
    pub file_sha256: String,
    /// Pixels as stored, EXIF orientation not applied.
    pub width: u32,
    /// Pixels as stored.
    pub height: u32,
    /// How the image maps to the world.
    pub projection: Projection,
    /// Whether the left and right edges meet.
    pub topology: Topology,
    /// The canonical pixel digest (2.2), lowercase hex.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pixel_sha256: Option<String>,
}

/// An image's projection (2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Projection {
    /// An ordinary picture.
    Flat,
    /// A 360-degree panorama, always `wrap`.
    Equirectangular,
}

/// An image's topology (2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Topology {
    /// Four edges.
    Bounded,
    /// x = 0 and x = 1 are the same edge.
    Wrap,
}

/// Where something is, on the original image (3).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Location {
    /// The image the location is on.
    pub image: ImageReference,
    /// The shape's kind (3.2).
    pub kind: LocationKind,
    /// The shape, in one part or, across the wrap edge, several.
    pub parts: Vec<Part>,
    /// True when one shape was split at the wrap edge.
    pub straddles_wrap: bool,
    /// What is there, in words.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// The producing tool's word for what the location means.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disposition: Option<String>,
    /// The producing tool's word for how sure it is.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence: Option<String>,
    /// The producing tool's word for the evidence behind it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence: Option<String>,
    /// A measured number in [0, 1]; never comparable across tools or kinds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub score: Option<f64>,
    /// What `score` is, such as `owlv2-detection`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub score_kind: Option<String>,
    /// Why; required for `absent`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// The tool, version, run and item that produced it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<Source>,
}

/// A location's kind (3.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LocationKind {
    /// An axis-aligned box.
    Bbox,
    /// A simple polygon of 3 to 256 vertices.
    Polygon,
    /// A column or a path with a width in pixels.
    Line,
    /// A brush path with a radius in pixels.
    Stroke,
    /// A greyscale rendering and a threshold.
    Mask,
    /// The whole image: a global defect.
    WholeImage,
    /// Missing; no geometry.
    Absent,
    /// A panorama view direction; input only.
    View,
}

/// One part of a location's shape (3.2). Which variant is allowed depends on the kind.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Part {
    /// `bbox`.
    Bbox(BboxPart),
    /// `polygon`.
    Polygon(PolygonPart),
    /// `line` along a column.
    Column(ColumnPart),
    /// `line` along a path.
    Path(PathPart),
    /// `stroke`.
    Stroke(StrokePart),
    /// `mask`.
    Mask(MaskPart),
    /// `view`.
    View(ViewPart),
}

/// A point as fractions `[x, y]` (3.1).
pub type Point = [f64; 2];

/// `[x0, y0, x1, y1]`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BboxPart {
    /// `[x0, y0, x1, y1]`, `x0 < x1`, `y0 < y1`.
    pub bbox: [f64; 4],
}

/// A simple polygon.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolygonPart {
    /// 3 to 256 vertices.
    pub polygon: Vec<Point>,
}

/// A vertical line.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ColumnPart {
    /// The column, as a fraction of the width.
    pub column: f64,
    /// Width in pixels of the referenced image; positive.
    pub width_px: f64,
}

/// A line along a path.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PathPart {
    /// 2 to 256 points.
    pub path: Vec<Point>,
    /// Width in pixels of the referenced image; positive.
    pub width_px: f64,
}

/// A brush stroke.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StrokePart {
    /// 2 to 256 points.
    pub path: Vec<Point>,
    /// Radius in pixels of the referenced image; positive.
    pub radius_px: f64,
}

/// A mask, crossing as its rendering.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MaskPart {
    /// The rendering and its threshold.
    pub mask: MaskRendering,
}

/// A greyscale 8-bit rendering at the located image's width and height.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MaskRendering {
    /// The rendering's reference.
    pub image: ImageReference,
    /// Pixels at or above it are inside; 1 to 255.
    pub threshold: u8,
}

/// A view direction on a panorama; input only.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ViewPart {
    /// Degrees from the centre column, increasing to the right.
    pub yaw: f64,
    /// Degrees from the horizon, increasing upward.
    pub pitch: f64,
    /// The horizontal field of a square view, 10 to 120 degrees.
    pub fov: f64,
}

/// What produced a location.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    /// The tool's name.
    pub tool: String,
    /// The tool's version.
    pub version: String,
    /// The run's id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run: Option<String>,
    /// The tool's id for the item within the run.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub item: Option<String>,
}

/// An image given to an action's slot (4.1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ImageIn {
    /// By absolute path; local transports only.
    Path(PathIn),
    /// Inline bytes.
    Inline(InlineIn),
}

/// An image given by path.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PathIn {
    /// An absolute path inside the client's roots.
    pub path: String,
    /// The reference the tool must verify.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expect: Option<ImageReference>,
}

/// An image given inline.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InlineIn {
    /// The bytes.
    pub inline: Inline,
    /// The reference the tool must verify.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expect: Option<ImageReference>,
}

/// Base64 image bytes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Inline {
    /// `image/png`, `image/jpeg`, ….
    pub media_type: String,
    /// Standard base64.
    pub data: String,
}

/// An image a tool returns (4.2).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImageOut {
    /// The role, from the tool's description.
    pub role: String,
    /// The image's reference.
    pub image: ImageReference,
    /// Where to get it.
    #[serde(flatten)]
    pub locator: Locator,
}

/// Where to get a returned image: exactly one.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Locator {
    /// An absolute path, on a local transport.
    Path(String),
    /// The bytes.
    Inline(Inline),
    /// An MCP resource URI.
    Resource(String),
}

/// A run's result (5): the MCP result's `structuredContent`, the run resource and `run.json`.
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
    /// How it ended (5.1).
    pub status: Status,
    /// Present exactly when `status` is `error` or `invalid` (5.2).
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
    /// The images taken in and returned.
    pub images: Images,
    /// The locations taken in and produced.
    pub locations: Locations,
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

/// How a run ended (5.1).
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

/// What went wrong (5.2).
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

/// The error classes (5.2).
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

/// The images a run took in and returned.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Images {
    /// Taken in.
    #[serde(rename = "in")]
    pub taken: Vec<ImageTaken>,
    /// Returned.
    pub out: Vec<ImageOut>,
}

/// An image a run took in.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImageTaken {
    /// The slot it came through.
    pub slot: String,
    /// Its role.
    pub role: String,
    /// Its reference.
    pub image: ImageReference,
}

/// The locations a run took in and produced.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Locations {
    /// Taken in.
    pub used: Vec<Location>,
    /// Produced.
    pub found: Vec<Location>,
}

/// The description document (6).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Description {
    /// The contract version the tool conforms to.
    pub contract: String,
    /// The tool.
    pub tool: DescribedTool,
    /// One per MCP tool, `describe` excepted.
    pub actions: Vec<Action>,
    /// Every word the tool emits in a location.
    pub vocabulary: Vocabulary,
    /// Size limits.
    pub limits: Limits,
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

/// One action (6).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Action {
    /// The MCP tool name.
    pub name: String,
    /// A short title.
    pub title: String,
    /// What it does, in a sentence.
    pub purpose: String,
    /// Image slots.
    pub images: Vec<ImageSlot>,
    /// Location slots.
    pub locations: Vec<LocationSlot>,
    /// A JSON Schema for the other arguments.
    pub params_schema: serde_json::Value,
    /// What it returns.
    pub outputs: Outputs,
    /// Whether and how it spends.
    pub cost: Cost,
    /// How long it takes.
    pub duration: Duration,
    /// Commands that render a run.
    pub renders: Vec<Render>,
}

/// An image slot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImageSlot {
    /// The argument name.
    pub slot: String,
    /// The role an image in it plays.
    pub role: String,
    /// Whether the call must fill it.
    pub required: bool,
}

/// A location slot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LocationSlot {
    /// The argument name.
    pub slot: String,
    /// The kinds it accepts.
    pub kinds: Vec<LocationKind>,
    /// Whether the call must fill it.
    pub required: bool,
}

/// What an action returns.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Outputs {
    /// The roles of returned images.
    pub images: Vec<String>,
    /// Which location lists it fills.
    pub locations: Vec<LocationOutput>,
}

/// A location list an action fills.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LocationOutput {
    /// `locations.found`.
    Found,
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

/// The words a tool emits in a location (3.2).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Vocabulary {
    /// `disposition` words.
    pub disposition: Vec<String>,
    /// `confidence` words.
    pub confidence: Vec<String>,
    /// `evidence` words.
    pub evidence: Vec<String>,
    /// `score_kind` words.
    pub score_kind: Vec<String>,
}

/// Size limits.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Limits {
    /// The largest inline image accepted, in decoded bytes.
    pub inline_bytes: u64,
}

/// The `<tool>://runs` resource (7.3).
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
