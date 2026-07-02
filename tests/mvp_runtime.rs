use juky4::kernel::classifier::ProjectType;
use juky4::kernel::ir::{compile_ir, ProjectIR};
use juky4::kernel::pipeline::{compile_repo_plan, detect_project_type};
use juky4::kernel::runtime::execute::{execute_node, execute_pglite, ExecutionStatus};
use juky4::kernel::runtime::selector::{select_runtime, RuntimeBackend};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn create_temp_repo_with_package_json(contents: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "juky4-integration-test-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should move forward")
            .as_nanos()
    ));
    fs::create_dir_all(&dir).expect("temp dir should be created");
    fs::write(dir.join("package.json"), contents).expect("package.json should be created");
    dir
}

#[test]
fn compiles_next_repo_plan_with_expected_commands() {
    let dir = create_temp_repo_with_package_json(r#"{"dependencies":{"next":"14.0.0"}}"#);
    let ir = compile_repo_plan(dir.to_str().unwrap_or_default());

    assert_eq!(ir.project_type, ProjectType::NextJs);
    assert_eq!(ir.install_cmd, "npm install");
    assert_eq!(ir.dev_cmd, "npm run dev");
    assert_eq!(ir.build_cmd.as_deref(), Some("npm run build && npm start"));

    fs::remove_dir_all(dir).expect("temp dir should be removed");
}

#[test]
fn selects_pglite_runtime_for_pglite_plan() {
    let dir = create_temp_repo_with_package_json(r#"{"dependencies":{"@electric-sql/pglite":"0.2.0"}}"#);
    let project_type = detect_project_type(dir.to_str().unwrap_or_default());
    let runtime = select_runtime(&project_type);

    assert_eq!(project_type, ProjectType::PGLiteApp);
    assert_eq!(runtime, RuntimeBackend::Pglite);

    fs::remove_dir_all(dir).expect("temp dir should be removed");
}

#[test]
fn execute_node_emits_trace_data() {
    let ir = ProjectIR {
        project_type: ProjectType::CreateReactApp,
        install_cmd: "true".into(),
        dev_cmd: "true".into(),
        build_cmd: Some("true".into()),
    };

    let result = execute_node(&ir);
    assert_eq!(result.runtime, "Node");
    assert_eq!(result.url.as_deref(), Some("http://localhost:3000"));
    assert_eq!(result.status, ExecutionStatus::Running);
    assert!(result.logs.iter().any(|line| line.contains("installing: true")));
}

#[test]
fn execute_pglite_emits_trace_data() {
    let ir = compile_ir(ProjectType::PGLiteApp);
    let ir = ProjectIR {
        dev_cmd: "true".into(),
        ..ir
    };

    let result = execute_pglite(&ir);
    assert_eq!(result.runtime, "Pglite");
    assert_eq!(result.url.as_deref(), Some("pglite://local-db"));
    assert_eq!(result.status, ExecutionStatus::Running);
    assert!(
        result
            .logs
            .iter()
            .any(|line| line.contains("starting PGLite runtime"))
    );
}
