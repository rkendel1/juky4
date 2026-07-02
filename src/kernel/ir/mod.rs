use crate::kernel::classifier::ProjectType;

#[derive(Debug, Clone)]
pub struct ProjectIR {
    pub project_type: ProjectType,
    pub install_cmd: String,
    pub dev_cmd: String,
    pub build_cmd: Option<String>,
}

pub fn compile_ir(project_type: ProjectType) -> ProjectIR {
    match project_type {
        ProjectType::CreateReactApp => ProjectIR {
            project_type,
            install_cmd: "npm install".into(),
            dev_cmd: "npm start".into(),
            build_cmd: Some("npm run build".into()),
        },
        ProjectType::NextJs => ProjectIR {
            project_type,
            install_cmd: "npm install".into(),
            dev_cmd: "npm run dev".into(),
            build_cmd: Some("npm run build && npm start".into()),
        },
        ProjectType::PGLiteApp => ProjectIR {
            project_type,
            install_cmd: "npm install".into(),
            dev_cmd: "node server.js".into(),
            build_cmd: None,
        },
        ProjectType::Unknown => ProjectIR {
            project_type,
            install_cmd: "echo 'unknown project'".into(),
            dev_cmd: "exit 1".into(),
            build_cmd: None,
        },
    }
}
