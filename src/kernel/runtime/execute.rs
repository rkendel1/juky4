use crate::kernel::ir::ProjectIR;
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionStatus {
    Running,
    Failed,
}

#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub project_type: String,
    pub runtime: String,
    pub logs: Vec<String>,
    pub url: Option<String>,
    pub status: ExecutionStatus,
}

pub fn execute_node(ir: &ProjectIR) -> ExecutionResult {
    let mut logs = vec![format!("installing: {}", ir.install_cmd)];

    let install_ok = Command::new("sh")
        .arg("-c")
        .arg(&ir.install_cmd)
        .status()
        .is_ok_and(|status| status.success());

    logs.push(format!("starting: {}", ir.dev_cmd));

    let started = Command::new("sh").arg("-c").arg(&ir.dev_cmd).spawn().is_ok();
    let status = if install_ok && started {
        ExecutionStatus::Running
    } else {
        ExecutionStatus::Failed
    };

    ExecutionResult {
        project_type: format!("{:?}", ir.project_type),
        runtime: "Node".into(),
        logs,
        url: Some("http://localhost:3000".into()),
        status,
    }
}

pub fn execute_pglite(ir: &ProjectIR) -> ExecutionResult {
    let mut logs = vec!["starting PGLite runtime".into()];
    logs.push(format!("executing: {}", ir.dev_cmd));

    let ran = Command::new("sh")
        .arg("-c")
        .arg(&ir.dev_cmd)
        .status()
        .is_ok_and(|status| status.success());

    let status = if ran {
        ExecutionStatus::Running
    } else {
        ExecutionStatus::Failed
    };

    ExecutionResult {
        project_type: format!("{:?}", ir.project_type),
        runtime: "Pglite".into(),
        logs,
        url: Some("pglite://local-db".into()),
        status,
    }
}
