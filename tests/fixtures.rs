//! Each tool's conformance fixtures (CONTRACT.md 7, item 7) are valid against the schemas and
//! read into the Rust types: `fixtures/<tool>/describe.json` and every `describe-*.json` a
//! description, every `estimate-*.json` an estimate, every `run-*.json` a result, every
//! `runs*.json` a runs page, `session.json` and every `session-*.json` a session, every
//! `refusal-*.json` a refusal's data; for the operations profile every `artifact-*.json` an
//! artifact resource, every `export-*.json` and `import-*.json` an export or import response, and
//! each of these three a refusal's data instead when it carries `reason`; every `bundle-*/`
//! directory an invocation bundle whose `invocation.json` is a manifest and whose every listed
//! member exists beside it with the digest the manifest states. Every dollar amount has at most
//! six decimals (4.6). Any other file fails. The tool generates them from its own code and keeps
//! its copy current in its own CI; here they are held to the contract.

use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};
use table_manners::operations::{ArtifactResource, ExportResponse, ImportResponse};
use table_manners::{
    CONTRACT_VERSION, Description, Estimate, InvocationBundle, Refusal, RunResult, RunsList,
    SCHEMAS, Session,
};

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

fn validator(id: &str) -> jsonschema::Validator {
    let (_, text) = SCHEMAS.iter().find(|(s, _)| *s == id).expect("schema");
    let schema: serde_json::Value = serde_json::from_str(text).expect("parses");
    jsonschema::options()
        .with_retriever(Embedded)
        .build(&schema)
        .expect("builds")
}

fn read(path: &Path) -> serde_json::Value {
    serde_json::from_str(&std::fs::read_to_string(path).expect("fixture")).expect("JSON")
}

/// Every dollar amount (`usd` or a key ending `_usd`) has at most six decimals (4.6), which the
/// schemas cannot say.
fn six_decimals(path: &Path, key: &str, value: &serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            for (k, v) in map {
                six_decimals(path, k, v);
            }
        }
        serde_json::Value::Array(items) => {
            for v in items {
                six_decimals(path, key, v);
            }
        }
        serde_json::Value::Number(n) if key == "usd" || key.ends_with("_usd") => {
            // Six decimals is a property of the amount, not of how it is spelled: serde_json
            // writes 0.000001 as `1e-6`, which is exactly six decimals. The amount in millionths
            // of a dollar must be a whole number.
            let usd = n.as_f64().expect("a finite amount");
            let micros = usd * 1_000_000.0;
            assert!(
                (micros - micros.round()).abs() < 1e-6,
                "{}: {key} is {n}, more than six decimals",
                path.display()
            );
        }
        _ => {}
    }
}

fn valid(validator: &jsonschema::Validator, path: &Path, doc: &serde_json::Value) {
    let errors: Vec<String> = validator.iter_errors(doc).map(|e| e.to_string()).collect();
    assert!(errors.is_empty(), "{}: {errors:?}", path.display());
}

/// Every contract version in a document (a bundle carries one per step and per result) is this
/// crate's.
fn at_this_version(path: &Path, value: &serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            for (k, v) in map {
                if k == "contract" {
                    assert_eq!(
                        v,
                        CONTRACT_VERSION,
                        "{}: fixtures are at this contract's version",
                        path.display()
                    );
                } else {
                    at_this_version(path, v);
                }
            }
        }
        serde_json::Value::Array(items) => {
            for v in items {
                at_this_version(path, v);
            }
        }
        _ => {}
    }
}

fn sha256_hex(path: &Path) -> String {
    let bytes = std::fs::read(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    Sha256::digest(&bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// A `bundle-*/` directory: `invocation.json` is a manifest, and every member it lists exists
/// beside it with the digest it states (an input's `file_sha256`, an artifact's `sha256`).
fn bundle(validator: &jsonschema::Validator, dir: &Path) {
    let manifest = dir.join("invocation.json");
    assert!(manifest.exists(), "{}: no invocation.json", dir.display());
    let doc = read(&manifest);
    six_decimals(&manifest, "", &doc);
    at_this_version(&manifest, &doc);
    valid(validator, &manifest, &doc);
    let b: InvocationBundle = serde_json::from_value(doc).expect("an InvocationBundle");
    for step in &b.steps {
        let members = step
            .inputs
            .iter()
            .map(|i| (&i.member, &i.image.file_sha256))
            .chain(step.artifacts.iter().map(|a| (&a.member, &a.sha256)));
        for (member, digest) in members {
            let path = dir.join(member);
            assert!(path.is_file(), "{}: member {member} missing", dir.display());
            assert_eq!(
                &sha256_hex(&path),
                digest,
                "{}: member {member} is not the bytes the manifest names",
                dir.display()
            );
        }
    }
}

#[test]
fn every_tools_fixtures_hold_to_the_contract() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures");
    let description = validator("urn:table-manners:core:description");
    let result = validator("urn:table-manners:core:result");
    let runs = validator("urn:table-manners:core:runs");
    let session = validator("urn:table-manners:core:session");
    let refusal = validator("urn:table-manners:core:refusal");
    let estimate = validator("urn:table-manners:core:estimate");
    let manifest = validator("urn:table-manners:core:invocation-bundle");
    let artifact = validator("urn:table-manners:operations:artifact");
    let export = validator("urn:table-manners:operations:export-response");
    let import = validator("urn:table-manners:operations:import-response");
    let mut tools: Vec<PathBuf> = std::fs::read_dir(&root)
        .expect("fixtures/")
        .map(|e| e.expect("entry").path())
        .filter(|p| p.is_dir())
        .collect();
    tools.sort();
    assert!(!tools.is_empty(), "no tool has fixtures yet");
    for tool in tools {
        let name = tool
            .file_name()
            .expect("name")
            .to_string_lossy()
            .into_owned();
        let describe = tool.join("describe.json");
        assert!(describe.exists(), "{name}: no describe.json");
        let mut results = 0;
        for entry in std::fs::read_dir(&tool).expect("tool dir") {
            let path = entry.expect("entry").path();
            let file = path
                .file_name()
                .expect("name")
                .to_string_lossy()
                .into_owned();
            if path.is_dir() {
                assert!(
                    file.starts_with("bundle-"),
                    "{}: not a fixture this test knows",
                    path.display()
                );
                bundle(&manifest, &path);
                continue;
            }
            let doc = read(&path);
            six_decimals(&path, "", &doc);
            at_this_version(&path, &doc);
            let refused = doc.get("reason").is_some();
            if file == "describe.json" || file.starts_with("describe-") {
                valid(&description, &path, &doc);
                let d: Description = serde_json::from_value(doc).expect("a Description");
                assert_eq!(d.tool.name, name, "{}: the tool's own name", path.display());
            } else if file.starts_with("estimate-") {
                valid(&estimate, &path, &doc);
                let e: Estimate = serde_json::from_value(doc).expect("an Estimate");
                assert_eq!(e.tool.name, name, "{}: the tool's own name", path.display());
            } else if file.starts_with("run-") {
                valid(&result, &path, &doc);
                let r: RunResult = serde_json::from_value(doc).expect("a RunResult");
                assert_eq!(r.tool.name, name, "{}: the tool's own name", path.display());
                results += 1;
            } else if file.starts_with("runs") {
                valid(&runs, &path, &doc);
                let l: RunsList = serde_json::from_value(doc).expect("a RunsList");
                assert_eq!(l.tool.name, name, "{}: the tool's own name", path.display());
            } else if (file.starts_with("artifact-")
                || file.starts_with("export-")
                || file.starts_with("import-"))
                && refused
            {
                valid(&refusal, &path, &doc);
                let _: Refusal = serde_json::from_value(doc).expect("a Refusal");
            } else if file.starts_with("artifact-") {
                valid(&artifact, &path, &doc);
                let a: ArtifactResource = serde_json::from_value(doc).expect("an ArtifactResource");
                assert_eq!(a.tool.name, name, "{}: the tool's own name", path.display());
            } else if file.starts_with("export-") {
                valid(&export, &path, &doc);
                let e: ExportResponse = serde_json::from_value(doc).expect("an ExportResponse");
                assert_eq!(e.tool.name, name, "{}: the tool's own name", path.display());
            } else if file.starts_with("import-") {
                valid(&import, &path, &doc);
                let i: ImportResponse = serde_json::from_value(doc).expect("an ImportResponse");
                assert_eq!(i.tool.name, name, "{}: the tool's own name", path.display());
            } else if file == "session.json" || file.starts_with("session-") {
                valid(&session, &path, &doc);
                let s: Session = serde_json::from_value(doc).expect("a Session");
                assert_eq!(s.tool.name, name, "{}: the tool's own name", path.display());
            } else if file.starts_with("refusal-") {
                valid(&refusal, &path, &doc);
                let _: Refusal = serde_json::from_value(doc).expect("a Refusal");
            } else {
                panic!("{}: not a fixture this test knows", path.display());
            }
        }
        assert!(results > 0, "{name}: no run-*.json");
    }
}
