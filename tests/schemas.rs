//! The schemas against the examples, and the Rust types against both.
//!
//! - Every `examples/<schema>/valid/*.json` validates and every `invalid/*.json` does not.
//! - Every valid example deserialises into its Rust type and serialises back to the same JSON, so
//!   the types and the schemas describe one shape.
//! - [`SCHEMAS`] lists exactly the files under `schemas/core/` and `schemas/images/`.

use std::path::{Path, PathBuf};

use serde::Serialize;
use serde::de::DeserializeOwned;
use table_manners::images::{ImageIn, ImageOut, ImageReference, Location};
use table_manners::{
    Description, Estimate, Refusal, RunRef, RunResult, RunsList, SCHEMAS, Session,
};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

/// Resolves `urn:table-manners:<dir>:<name>` from [`SCHEMAS`], never from the network.
struct Embedded;

impl jsonschema::Retrieve for Embedded {
    fn retrieve(
        &self,
        uri: &jsonschema::Uri<String>,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        let text = SCHEMAS
            .iter()
            .find(|(id, _)| *id == uri.as_str())
            .map(|(_, text)| *text)
            .ok_or_else(|| format!("no embedded schema {uri}"))?;
        Ok(serde_json::from_str(text)?)
    }
}

/// The validator for the schema named `name`, in whichever directory it is.
fn validator(name: &str) -> jsonschema::Validator {
    let (id, text) = SCHEMAS
        .iter()
        .find(|(id, _)| id.rsplit(':').next() == Some(name))
        .unwrap_or_else(|| panic!("no schema named {name}"));
    let schema: serde_json::Value = serde_json::from_str(text).expect("schema parses");
    jsonschema::options()
        .with_retriever(Embedded)
        .build(&schema)
        .unwrap_or_else(|e| panic!("{id}: {e}"))
}

fn examples(name: &str, which: &str) -> Vec<(String, serde_json::Value)> {
    let dir = root().join("examples").join(name).join(which);
    let mut out: Vec<_> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("{}: {e}", dir.display()))
        .map(|entry| entry.expect("entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .map(|path| {
            let text = std::fs::read_to_string(&path).expect("example");
            let file = path
                .file_name()
                .expect("name")
                .to_string_lossy()
                .into_owned();
            (file, serde_json::from_str(&text).expect("example parses"))
        })
        .collect();
    out.sort_by(|a, b| a.0.cmp(&b.0));
    assert!(!out.is_empty(), "{}: no examples", dir.display());
    out
}

/// `24` and `24.0` are one JSON number; serde_json's `Value` tells them apart.
fn numbers_as_floats(value: serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Number(n) => n
            .as_f64()
            .and_then(serde_json::Number::from_f64)
            .map_or(serde_json::Value::Null, serde_json::Value::Number),
        serde_json::Value::Array(items) => {
            serde_json::Value::Array(items.into_iter().map(numbers_as_floats).collect())
        }
        serde_json::Value::Object(map) => serde_json::Value::Object(
            map.into_iter()
                .map(|(k, v)| (k, numbers_as_floats(v)))
                .collect(),
        ),
        other => other,
    }
}

fn check<T: DeserializeOwned + Serialize>(name: &str) {
    let validator = validator(name);
    for (file, doc) in examples(name, "valid") {
        let errors: Vec<String> = validator.iter_errors(&doc).map(|e| e.to_string()).collect();
        assert!(errors.is_empty(), "{name}/valid/{file}: {errors:?}");
        let typed: T = serde_json::from_value(doc.clone())
            .unwrap_or_else(|e| panic!("{name}/valid/{file}: the type rejects it: {e}"));
        let back = serde_json::to_value(&typed).expect("serialises");
        assert_eq!(
            numbers_as_floats(back),
            numbers_as_floats(doc),
            "{name}/valid/{file}: the type changes it"
        );
    }
    for (file, doc) in examples(name, "invalid") {
        assert!(
            !validator.is_valid(&doc),
            "{name}/invalid/{file}: the schema accepts it"
        );
    }
}

#[test]
fn image_reference() {
    check::<ImageReference>("image-reference");
}

#[test]
fn location() {
    check::<Location>("location");
}

#[test]
fn image_in() {
    check::<ImageIn>("image-in");
}

#[test]
fn image_out() {
    check::<ImageOut>("image-out");
}

#[test]
fn result() {
    check::<RunResult>("result");
}

#[test]
fn description() {
    check::<Description>("description");
}

#[test]
fn runs() {
    check::<RunsList>("runs");
}

#[test]
fn run_ref() {
    check::<RunRef>("run-ref");
}

#[test]
fn session() {
    check::<Session>("session");
}

#[test]
fn refusal() {
    check::<Refusal>("refusal");
}

#[test]
fn estimate() {
    check::<Estimate>("estimate");
}

#[test]
fn the_embedded_schemas_are_the_files_under_schemas() {
    let mut files: Vec<String> = Vec::new();
    for dir in ["core", "images"] {
        for entry in std::fs::read_dir(root().join("schemas").join(dir)).expect("schemas/<dir>/") {
            let name = entry
                .expect("entry")
                .file_name()
                .to_string_lossy()
                .into_owned();
            files.push(format!("{dir}/{name}"));
        }
    }
    files.sort();
    let mut embedded: Vec<String> = SCHEMAS
        .iter()
        .map(|(id, _)| {
            let rest = id
                .strip_prefix("urn:table-manners:")
                .expect("an id in the contract's namespace");
            let (dir, name) = rest.split_once(':').expect("<dir>:<name>");
            format!("{dir}/{name}.json")
        })
        .collect();
    embedded.sort();
    assert_eq!(files, embedded);
    for (id, text) in SCHEMAS {
        let schema: serde_json::Value = serde_json::from_str(text).expect("parses");
        assert_eq!(
            schema["$id"], *id,
            "each schema's $id is its directory and name"
        );
        assert_eq!(
            schema["$schema"],
            "https://json-schema.org/draft/2020-12/schema"
        );
    }
}

#[test]
fn every_example_directory_has_a_test() {
    let mut dirs: Vec<String> = std::fs::read_dir(root().join("examples"))
        .expect("examples/")
        .map(|entry| {
            entry
                .expect("entry")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    dirs.sort();
    assert_eq!(
        dirs,
        [
            "description",
            "estimate",
            "image-in",
            "image-out",
            "image-reference",
            "location",
            "refusal",
            "result",
            "run-ref",
            "runs",
            "session"
        ]
    );
}
