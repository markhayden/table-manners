//! The images profile (CONTRACT.md part 2): image references, locations, images in and out, and
//! what the profile adds to the result and the description.

use serde::{Deserialize, Serialize};

/// How a tool names an image to another tool (8).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImageReference {
    /// SHA-256 of the file's bytes, lowercase hex; the enforced check (8.1).
    pub file_sha256: String,
    /// Pixels as stored, EXIF orientation not applied.
    pub width: u32,
    /// Pixels as stored.
    pub height: u32,
    /// How the image maps to the world.
    pub projection: Projection,
    /// Whether the left and right edges meet.
    pub topology: Topology,
    /// The canonical pixel digest (8.2), lowercase hex.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pixel_sha256: Option<String>,
}

/// An image's projection (8).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Projection {
    /// An ordinary picture.
    Flat,
    /// A 360-degree panorama, always `wrap`.
    Equirectangular,
}

/// An image's topology (8).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Topology {
    /// Four edges.
    Bounded,
    /// x = 0 and x = 1 are the same edge.
    Wrap,
}

/// Where something is, on the original image (9).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Location {
    /// The image the location is on.
    pub image: ImageReference,
    /// The shape's kind (9.2).
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

/// A location's kind (9.2).
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

/// One part of a location's shape (9.2). Which variant is allowed depends on the kind.
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

/// A point as fractions `[x, y]` (9.1).
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

/// An image given to an action's slot (10.1).
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

/// An image a tool returns (10.2).
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
    /// Whether it takes a non-empty list of images (10.1); false when absent.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub many: bool,
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

/// The words a tool emits in a location (9.2).
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
