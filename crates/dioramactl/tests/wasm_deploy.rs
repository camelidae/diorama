use dioramactl::{Command, ParseError, parse_command};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

fn args(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).into()).collect()
}

fn tmpdir() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time")
        .as_nanos();
    std::env::temp_dir().join(format!("dioramactl-wasm-deploy-{nanos}"))
}

#[test]
fn parse_wasm_deploy_dry_run() {
    let parsed = parse_command(
        &args(&[
            "wasm",
            "deploy",
            "app.wasm",
            "--dry-run",
            "-o",
            "./dist/app",
            "--name",
            "demo",
            "--port",
            "9090",
        ]),
        &BTreeMap::new(),
    )
    .unwrap();
    let Command::Wasm(options) = parsed else {
        panic!("expected wasm command");
    };
    assert!(options.dry_run);
    assert_eq!(options.wasm, PathBuf::from("app.wasm"));
    assert_eq!(options.output.as_deref(), Some(Path::new("./dist/app")));
    assert_eq!(options.name.as_deref(), Some("demo"));
    assert_eq!(options.port, 9090);
}

#[test]
fn parse_wasm_deploy_without_dry_run() {
    let parsed = parse_command(
        &args(&["wasm", "deploy", "app.wasm", "-o", "./dist/app"]),
        &BTreeMap::new(),
    )
    .unwrap();
    let Command::Wasm(options) = parsed else {
        panic!("expected wasm command");
    };
    assert!(!options.dry_run);
}

#[test]
fn parse_wasm_missing_output_still_parses_with_dry_run() {
    let parsed = parse_command(
        &args(&["wasm", "deploy", "app.wasm", "--dry-run"]),
        &BTreeMap::new(),
    )
    .unwrap();
    let Command::Wasm(options) = parsed else {
        panic!("expected wasm command");
    };
    assert!(options.dry_run);
    assert!(options.output.is_none());
}

#[test]
fn parse_wasm_unknown_flag() {
    assert_eq!(
        parse_command(
            &args(&["wasm", "deploy", "app.wasm", "--dry-run", "--nope"]),
            &BTreeMap::new()
        ),
        Err(ParseError::UnknownFlag)
    );
}

#[test]
fn parse_wasm_help() {
    assert!(matches!(
        parse_command(&args(&["wasm", "--help"]), &BTreeMap::new()),
        Ok(Command::Help(_))
    ));
}

#[test]
fn execute_dry_run_writes_artifacts() {
    let dir = tmpdir();
    fs::create_dir_all(&dir).unwrap();
    let wasm = dir.join("sample.wasm");
    fs::write(&wasm, b"\0asm\x01\x00\x00\x00").unwrap();
    let out = dir.join("dist");
    let wasm_path = wasm.to_string_lossy().into_owned();
    let out_path = out.to_string_lossy().into_owned();
    let parsed = parse_command(
        &args(&[
            "wasm",
            "deploy",
            &wasm_path,
            "--dry-run",
            "-o",
            &out_path,
            "--name",
            "sample",
        ]),
        &BTreeMap::new(),
    )
    .unwrap();
    let Command::Wasm(options) = parsed else {
        panic!("expected wasm command");
    };
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    assert_eq!(
        dioramactl::wasm::execute(&options, &mut stdout, &mut stderr).unwrap(),
        0
    );
    for name in [
        "app.wasm",
        "app.image.tar",
        "wasm-app-job.yml",
        "wasm-app-storage.yml",
    ] {
        assert!(out.join(name).is_file(), "missing {name}");
    }
    let job = fs::read_to_string(out.join("wasm-app-job.yml")).unwrap();
    assert!(job.contains("kind: Job"));
    assert!(job.contains("restartPolicy: Never"));
    assert!(job.contains("runtimeClassName: wasmtime"));
    assert!(job.contains("image: oci-registry:5000/sample:dev"));
    let tar = fs::read(out.join("app.image.tar")).unwrap();
    let tar_text = String::from_utf8_lossy(&tar);
    assert!(tar_text.contains("oci-layout"));
    assert!(tar_text.contains("index.json"));
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn execute_without_output_errors() {
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = dioramactl::execute(
        &args(&["wasm", "deploy", "app.wasm"]),
        &BTreeMap::new(),
        "0.0.0-test",
        &mut NoopInputs,
        &mut stdout,
        &mut stderr,
    )
    .unwrap();
    assert_eq!(code, 1);
    assert!(String::from_utf8_lossy(&stderr).contains("--output"));
}

#[test]
fn parse_wasm_kind_deployment() {
    let parsed = parse_command(
        &args(&[
            "wasm",
            "deploy",
            "app.wasm",
            "-o",
            "./dist/app",
            "--kind",
            "deployment",
        ]),
        &BTreeMap::new(),
    )
    .unwrap();
    let Command::Wasm(options) = parsed else {
        panic!("expected wasm command");
    };
    assert_eq!(options.kind, dioramactl::wasm::WorkloadKind::Deployment);
}

#[test]
fn parse_wasm_registry_flag() {
    let parsed = parse_command(
        &args(&[
            "wasm",
            "deploy",
            "app.wasm",
            "-o",
            "./dist/app",
            "--registry",
            "http://127.0.0.1:34521",
            "--push-only",
        ]),
        &BTreeMap::new(),
    )
    .unwrap();
    let Command::Wasm(options) = parsed else {
        panic!("expected wasm command");
    };
    assert_eq!(options.registry.as_deref(), Some("http://127.0.0.1:34521"));
    assert!(options.push_only);
}

#[test]
fn execute_dry_run_without_output_errors() {
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = dioramactl::execute(
        &args(&["wasm", "deploy", "app.wasm", "--dry-run"]),
        &BTreeMap::new(),
        "0.0.0-test",
        &mut NoopInputs,
        &mut stdout,
        &mut stderr,
    )
    .unwrap();
    assert_eq!(code, 1);
    assert!(String::from_utf8_lossy(&stderr).contains("--output"));
}

struct NoopInputs;

impl dioramactl::CheckInputs for NoopInputs {
    fn discover(
        &mut self,
    ) -> Result<diorama_platform::HostEvidence, diorama_platform::PlatformError> {
        unreachable!("wasm deploy does not discover host evidence")
    }

    fn supplemental(
        &mut self,
    ) -> Result<diorama_platform::preflight_probe::SupplementalFacts, diorama_platform::PlatformError>
    {
        unreachable!("wasm deploy does not collect supplemental facts")
    }

    fn ports(
        &mut self,
        _pprof: bool,
    ) -> Result<
        [diorama_platform::Observation<diorama_platform::preflight::PortAvailability>; 4],
        diorama_platform::PlatformError,
    > {
        unreachable!("wasm deploy does not probe ports")
    }
}
