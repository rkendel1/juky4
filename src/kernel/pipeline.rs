use crate::kernel::classifier::{classify_repo, ProjectType};
use crate::kernel::ir::{compile_ir, ProjectIR};
use crate::kernel::runtime::execute::{execute_node, execute_pglite, ExecutionResult};
use crate::kernel::runtime::selector::{select_runtime, RuntimeBackend};

pub fn compile_repo_plan(repo_path: &str) -> ProjectIR {
    let project_type = classify_repo(repo_path);
    compile_ir(project_type)
}

pub fn run_repo(repo_path: &str) -> ExecutionResult {
    let ir = compile_repo_plan(repo_path);
    let runtime = select_runtime(&ir.project_type);

    match runtime {
        RuntimeBackend::Node => execute_node(&ir),
        RuntimeBackend::Pglite => execute_pglite(&ir),
    }
}

pub fn detect_project_type(repo_path: &str) -> ProjectType {
    classify_repo(repo_path)
}
