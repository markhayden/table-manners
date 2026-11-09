//! Each tool's conformance fixtures (CONTRACT.md 7, item 7) are valid against the schemas and
//! read into the Rust types: `fixtures/<tool>/describe.json` and every `describe-*.json` a
//! description, every `estimate-*.json` an estimate, every `run-*.json` a result, every
//! `runs*.json` a runs page, `session.json` and every `session-*.json` a session, every
//! `refusal-*.json` a refusal's data; for the operations profile every `artifact-*.json` an
//! artifact resource, every `export-*.json` and `import-*.json` an export or import response, and
//! each of these three a refusal's data instead when it carries `reason`; every `install-*.json`
//! and `uninstall-*.json` an install control's result, or a refusal's data when it carries
//! `reason` (5.3); every `bundle-*/`
//! directory an invocation bundle whose `invocation.json` is a manifest and whose every listed
//! member exists beside it with the digest the manifest states, and nothing else (4.9). Every dollar amount has at most
//! six decimals (4.6). Any other file fails.
//!
//! Beside the schemas, the rules they cannot say: in a description, port names are unique within
//! an action's `in` and its `out`, every port type is `image` or listed in `artifact_types`, each
//! artifact type is listed once, every `group` is in `vocabulary.group` and a tool's own type
//! carries its own name (3, 3.3, 4.7). In a result (and each bundled step's), `invocation.action`
//! is the result's action and no image in the invocation has a locator (4.8); an out artifact has
//! a preview exactly when its type declares `preview: true` (4.7); an `image` artifact's identity
//! is `sha256:` and the `file_sha256` of an image in the result's `images` (3.4, 4.7); and a
//! reference's own type carries the name of the tool that made it (3.3); and a result carries a
//! `verdict` only when its action declares `answers: verdict` in one of the tool's descriptions
//! (2.4). Per-choice readiness (3.1): a requirement's `for_choice` names a parameter whose `enum`
//! holds the value and whose `x-requirements[value]` names the requirement, every name listed in
//! `x-requirements` is a requirement of that choice, such a parameter has a `default` in its
//! `enum`, and the action's state follows the truth table (only the default's choice requirements
//! and the plain ones count); installable ids are unique and every requirement's `installable`
//! is one of them (3.5). The tool generates them from its own code and keeps
//! its copy current in its own CI; here they are held to the contract.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};
use table_manners::operations::{ArtifactResource, ExportResponse, ImportResponse};
use table_manners::{
    Answers, CONTRACT_VERSION, Description, Estimate, InstallResult, InvocationBundle,
    ReadinessState, Refusal, RequirementState, RunResult, RunsList, SCHEMAS, Session,
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

/// A tool's own type (`<tool>:<Name>`) carries the tool's name; shared types have no prefix.
fn own_type(path: &Path, artifact_type: &str, tool: &str) {
    if let Some((prefix, _)) = artifact_type.split_once(':') {
        assert_eq!(
            prefix,
            tool,
            "{}: type {artifact_type} is not {tool}'s own",
            path.display()
        );
    }
}

/// The description's rules no schema can say (3, 3.3, 4.7).
/// The truth table of 3.1: a requirement blocks when it is missing or invalid and is neither
/// optional nor a choice other than its parameter's default.
fn choice_rules(path: &Path, d: &Description) {
    let ids: Vec<&str> = d
        .installables
        .iter()
        .flatten()
        .map(|i| i.id.as_str())
        .collect();
    for (i, id) in ids.iter().enumerate() {
        assert!(
            !ids[..i].contains(id),
            "{}: installable {id} is listed twice",
            path.display()
        );
    }
    for a in &d.actions {
        let props = a.params_schema.get("properties");
        let mut blocking = false;
        for r in &a.readiness.requirements {
            if let Some(installable) = &r.installable {
                assert!(
                    ids.contains(&installable.as_str()),
                    "{}: {}'s requirement {} names installable {installable}, which is not listed",
                    path.display(),
                    a.name,
                    r.name
                );
            }
            let unmet = matches!(
                r.state,
                RequirementState::Missing | RequirementState::Invalid
            );
            let Some(choice) = &r.for_choice else {
                blocking |= unmet && !r.optional;
                continue;
            };
            let param = props.and_then(|p| p.get(&choice.param)).unwrap_or_else(|| {
                panic!(
                    "{}: {}'s requirement {} is for parameter {}, which params_schema lacks",
                    path.display(),
                    a.name,
                    r.name,
                    choice.param
                )
            });
            let in_enum = param["enum"]
                .as_array()
                .is_some_and(|e| e.iter().any(|v| v == choice.value.as_str()));
            assert!(
                in_enum,
                "{}: {}'s choice {}={} is not in the parameter's enum",
                path.display(),
                a.name,
                choice.param,
                choice.value
            );
            let listed = param["x-requirements"][choice.value.as_str()]
                .as_array()
                .is_some_and(|names| names.iter().any(|n| n == r.name.as_str()));
            assert!(
                listed,
                "{}: {}'s requirement {} is not in x-requirements[{}]",
                path.display(),
                a.name,
                r.name,
                choice.value
            );
            blocking |= unmet && param["default"] == choice.value.as_str();
        }
        for (name, param) in props.and_then(|p| p.as_object()).into_iter().flatten() {
            let Some(listed) = param.get("x-requirements") else {
                continue;
            };
            let choices = param["enum"].as_array().unwrap_or_else(|| {
                panic!(
                    "{}: {}.{name} has x-requirements but no enum",
                    path.display(),
                    a.name
                )
            });
            assert!(
                choices.contains(&param["default"]),
                "{}: {}.{name} has x-requirements but no default in its enum",
                path.display(),
                a.name
            );
            for (value, names) in listed.as_object().expect("x-requirements is an object") {
                assert!(
                    choices.iter().any(|c| c == value.as_str()),
                    "{}: {}.{name}'s x-requirements names {value}, which is not in its enum",
                    path.display(),
                    a.name
                );
                for n in names.as_array().expect("a list of names") {
                    let found = a.readiness.requirements.iter().any(|r| {
                        n.as_str() == Some(r.name.as_str())
                            && r.for_choice
                                .as_ref()
                                .is_some_and(|c| c.param == *name && c.value == *value)
                    });
                    assert!(
                        found,
                        "{}: {}.{name}={value} needs {n}, which readiness does not list for that choice",
                        path.display(),
                        a.name
                    );
                }
            }
        }
        match a.readiness.state {
            ReadinessState::Ready => assert!(
                !blocking,
                "{}: {} is ready but its default needs what is missing",
                path.display(),
                a.name
            ),
            ReadinessState::NeedsSetup => assert!(
                blocking,
                "{}: {} needs setup but nothing its default needs is missing",
                path.display(),
                a.name
            ),
            ReadinessState::Unavailable => {}
        }
    }
}

fn description_rules(path: &Path, d: &Description) {
    choice_rules(path, d);
    let types: Vec<&str> = d
        .artifact_types
        .iter()
        .flatten()
        .map(|t| t.artifact_type.as_str())
        .collect();
    for (i, t) in types.iter().enumerate() {
        assert!(
            !types[..i].contains(t),
            "{}: artifact type {t} is listed twice",
            path.display()
        );
        own_type(path, t, &d.tool.name);
    }
    let groups = d.vocabulary.as_ref().and_then(|v| v.group.as_ref());
    for a in &d.actions {
        if let Some(group) = &a.group {
            assert!(
                groups.is_some_and(|g| g.contains(group)),
                "{}: {}'s group {group} is not in vocabulary.group",
                path.display(),
                a.name
            );
        }
        let Some(ports) = &a.ports else { continue };
        for (side, list) in [("in", &ports.inputs), ("out", &ports.out)] {
            for (i, port) in list.iter().enumerate() {
                assert!(
                    !list[..i].iter().any(|p| p.name == port.name),
                    "{}: {} has two {side} ports named {}",
                    path.display(),
                    a.name,
                    port.name
                );
                let t = port.artifact_type.as_str();
                assert!(
                    t == "image" || types.contains(&t),
                    "{}: {}'s port {} names type {t}, which artifact_types does not list",
                    path.display(),
                    a.name,
                    port.name
                );
            }
        }
    }
}

/// No image inside an invocation's arguments has a locator (4.8): no object holds `inline`
/// bytes, and no object of only `path` and `expect` holds a `path`.
fn no_locator(path: &Path, value: &serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            let inline = map
                .get("inline")
                .is_some_and(|i| i.get("media_type").is_some());
            let by_path =
                map.contains_key("path") && map.keys().all(|k| k == "path" || k == "expect");
            assert!(
                !inline && !by_path,
                "{}: an image in the invocation has a locator",
                path.display()
            );
            for v in map.values() {
                no_locator(path, v);
            }
        }
        serde_json::Value::Array(items) => {
            for v in items {
                no_locator(path, v);
            }
        }
        _ => {}
    }
}

/// The result's rules no schema can say (3.3, 3.4, 4.7, 4.8). `description` is the tool's own,
/// when the result is that tool's.
fn result_rules(path: &Path, r: &RunResult, description: Option<&Description>) {
    if let Some(invocation) = &r.invocation {
        assert_eq!(
            invocation.action,
            r.action,
            "{}: the invocation is of another action",
            path.display()
        );
        no_locator(
            path,
            &serde_json::Value::Object(invocation.arguments.clone()),
        );
    }
    if let (Some(_), Some(d)) = (&r.verdict, description) {
        let declared = d
            .actions
            .iter()
            .any(|a| a.name == r.action && a.answers == Some(Answers::Verdict));
        assert!(
            declared,
            "{}: a verdict from {}, whose description does not declare answers: verdict",
            path.display(),
            r.action
        );
    }
    let Some(artifacts) = &r.artifacts else {
        return;
    };
    let files: Vec<&str> = r
        .images
        .iter()
        .flat_map(|i| {
            i.taken
                .iter()
                .map(|t| &t.image)
                .chain(i.out.iter().map(|o| &o.image))
        })
        .map(|image| image.file_sha256.as_str())
        .collect();
    for entry in artifacts.taken.iter().chain(&artifacts.out) {
        let a = &entry.artifact;
        own_type(path, &a.artifact_type, &a.tool);
        if a.artifact_type == "image" {
            let file = a.identity.strip_prefix("sha256:").unwrap_or_default();
            assert!(
                files.contains(&file),
                "{}: image artifact {} is not sha256: and the file_sha256 of an image in images",
                path.display(),
                a.identity
            );
        }
    }
    let Some(d) = description else { return };
    for entry in &artifacts.out {
        let declared = d
            .artifact_types
            .iter()
            .flatten()
            .any(|t| t.artifact_type == entry.artifact.artifact_type && t.preview);
        assert_eq!(
            entry.preview.is_some(),
            declared,
            "{}: out artifact {} on port {} has a preview exactly when its type declares one",
            path.display(),
            entry.artifact.identity,
            entry.port
        );
    }
}

/// A `bundle-*/` directory: `invocation.json` is a manifest, every member it lists exists beside
/// it with the digest it states (an input's `file_sha256`, an artifact's `sha256`), and the
/// directory holds nothing else (4.9).
fn bundle(validator: &jsonschema::Validator, dir: &Path, tool: &str, d: Option<&Description>) {
    let manifest = dir.join("invocation.json");
    assert!(manifest.exists(), "{}: no invocation.json", dir.display());
    let doc = read(&manifest);
    six_decimals(&manifest, "", &doc);
    at_this_version(&manifest, &doc);
    valid(validator, &manifest, &doc);
    let b: InvocationBundle = serde_json::from_value(doc).expect("an InvocationBundle");
    let mut listed = BTreeSet::new();
    for step in &b.steps {
        assert_eq!(
            step.invocation.action,
            step.result.action,
            "{}: a step's invocation is of another action",
            manifest.display()
        );
        let own = (step.result.tool.name == tool).then_some(d).flatten();
        result_rules(&manifest, &step.result, own);
        let members = step
            .inputs
            .iter()
            .map(|i| (&i.member, &i.image.file_sha256))
            .chain(step.artifacts.iter().map(|a| (&a.member, &a.sha256)));
        for (member, digest) in members {
            listed.insert(member.clone());
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
    let mut present = BTreeSet::new();
    files_under(dir, dir, &mut present);
    present.remove("invocation.json");
    assert_eq!(
        present,
        listed,
        "{}: the members are exactly the ones the manifest lists",
        dir.display()
    );
}

/// Every file under `dir`, as a `/`-separated path relative to `root`.
fn files_under(root: &Path, dir: &Path, out: &mut BTreeSet<String>) {
    for entry in std::fs::read_dir(dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display())) {
        let path = entry.expect("entry").path();
        if path.is_dir() {
            files_under(root, &path, out);
        } else {
            let rel = path.strip_prefix(root).expect("under root");
            let parts: Vec<String> = rel
                .components()
                .map(|c| c.as_os_str().to_string_lossy().into_owned())
                .collect();
            out.insert(parts.join("/"));
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
    let install = validator("urn:table-manners:core:install-result");
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
        // The tool's own description first: results are checked against its artifact types.
        let mut own: Option<Description> = serde_json::from_value(read(&describe)).ok();
        if let Some(d) = &own {
            description_rules(&describe, d);
        }
        // A tool's description follows its configuration (rascal's `remove_prompt` declares
        // `answers: verdict` only with a checker configured), so an action declares it when any
        // of the tool's descriptions does (2.4).
        if let Some(d) = own.as_mut() {
            for path in std::fs::read_dir(&tool).expect("tool dir") {
                let path = path.expect("entry").path();
                let file = path.file_name().expect("name").to_string_lossy();
                if !file.starts_with("describe-") {
                    continue;
                }
                let Ok(other) = serde_json::from_value::<Description>(read(&path)) else {
                    continue;
                };
                for judging in other
                    .actions
                    .iter()
                    .filter(|a| a.answers == Some(Answers::Verdict))
                {
                    for action in d.actions.iter_mut().filter(|a| a.name == judging.name) {
                        action.answers = Some(Answers::Verdict);
                    }
                }
            }
        }
        let mut results = 0;
        let mut entries: Vec<PathBuf> = std::fs::read_dir(&tool)
            .expect("tool dir")
            .map(|e| e.expect("entry").path())
            .collect();
        entries.sort();
        for path in entries {
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
                bundle(&manifest, &path, &name, own.as_ref());
                continue;
            }
            let doc = read(&path);
            six_decimals(&path, "", &doc);
            at_this_version(&path, &doc);
            let refused = doc.get("reason").is_some();
            if file == "describe.json" || file.starts_with("describe-") {
                valid(&description, &path, &doc);
                let d: Description = serde_json::from_value(doc).expect("a Description");
                description_rules(&path, &d);
                assert_eq!(d.tool.name, name, "{}: the tool's own name", path.display());
            } else if file.starts_with("estimate-") {
                valid(&estimate, &path, &doc);
                let e: Estimate = serde_json::from_value(doc).expect("an Estimate");
                assert_eq!(e.tool.name, name, "{}: the tool's own name", path.display());
            } else if file.starts_with("run-") {
                valid(&result, &path, &doc);
                let r: RunResult = serde_json::from_value(doc).expect("a RunResult");
                result_rules(&path, &r, own.as_ref());
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
            } else if (file.starts_with("install-") || file.starts_with("uninstall-")) && refused {
                valid(&refusal, &path, &doc);
                let _: Refusal = serde_json::from_value(doc).expect("a Refusal");
            } else if file.starts_with("install-") || file.starts_with("uninstall-") {
                valid(&install, &path, &doc);
                let r: InstallResult = serde_json::from_value(doc).expect("an InstallResult");
                let listed = own
                    .as_ref()
                    .and_then(|d| d.installables.as_ref())
                    .is_some_and(|list| list.iter().any(|i| i.id == r.id));
                assert!(
                    listed,
                    "{}: {} is not an installable of {name}'s description",
                    path.display(),
                    r.id
                );
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

/// The verdict rule refuses a result whose action does not declare `answers: verdict`, and
/// accepts the same result once it does (2.4). The examples are the schemas' own.
#[test]
fn a_verdict_needs_its_action_to_declare_answers() {
    let examples = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples");
    let result = examples.join("result/valid/verdict-pass.json");
    let r: RunResult = serde_json::from_value(read(&result)).expect("a RunResult");
    let declared: Description = serde_json::from_value(read(
        &examples.join("description/valid/verdict-and-optional-requirement.json"),
    ))
    .expect("a Description");
    assert_eq!(
        declared.actions[0].name, r.action,
        "the examples name one action"
    );
    result_rules(&result, &r, Some(&declared));
    let mut undeclared = declared.clone();
    undeclared.actions[0].answers = None;
    let refused = std::panic::catch_unwind(|| result_rules(&result, &r, Some(&undeclared)));
    assert!(
        refused.is_err(),
        "a verdict from an undeclared action is refused"
    );
}

/// Every valid description example keeps the rules a schema cannot say, and per-choice readiness
/// refuses each way a description can get it wrong (3.1, 3.5).
#[test]
fn per_choice_readiness_follows_the_truth_table() {
    let examples = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/description/valid");
    for entry in std::fs::read_dir(&examples).expect("examples") {
        let path = entry.expect("entry").path();
        let d: Description = serde_json::from_value(read(&path)).expect("a Description");
        description_rules(&path, &d);
    }
    let path = examples.join("choice-default-missing.json");
    let base: Description = serde_json::from_value(read(&path)).expect("a Description");
    let refused = |name: &str, change: &dyn Fn(&mut Description)| {
        let mut d = base.clone();
        change(&mut d);
        let outcome = std::panic::catch_unwind(|| description_rules(&path, &d));
        assert!(outcome.is_err(), "{name}: accepted");
    };
    // The default (sam3) is missing, so the action cannot be ready.
    refused("ready while the default is missing", &|d| {
        d.actions[0].readiness.state = ReadinessState::Ready;
    });
    // With the default installed, a missing other choice does not make it need setup.
    refused("needs setup while only another choice is missing", &|d| {
        d.actions[0].params_schema["properties"]["model"]["default"] = "tiny".into();
    });
    refused("a choice not in the enum", &|d| {
        d.actions[0].readiness.requirements[1]
            .for_choice
            .as_mut()
            .expect("a choice")
            .value = "hqsam".into();
    });
    refused("a choice's requirement not in x-requirements", &|d| {
        d.actions[0].readiness.requirements[1].name = "segmenter:other".into();
    });
    refused("x-requirements naming a choice outside the enum", &|d| {
        d.actions[0].params_schema["properties"]["model"]["x-requirements"]["hqsam"] =
            serde_json::json!(["segmenter:hqsam"]);
    });
    refused("x-requirements with no default", &|d| {
        let model = d.actions[0].params_schema["properties"]["model"]
            .as_object_mut()
            .expect("an object");
        model.remove("default");
    });
    refused("an installable the description does not list", &|d| {
        d.actions[0].readiness.requirements[0].installable = Some("hqsam".into());
    });
    refused("an installable listed twice", &|d| {
        let again = d.installables.as_ref().expect("installables")[0].clone();
        d.installables.as_mut().expect("installables").push(again);
    });
}
