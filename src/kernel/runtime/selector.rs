use crate::kernel::classifier::ProjectType;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeBackend {
    Node,
    Pglite,
}

pub fn select_runtime(project_type: &ProjectType) -> RuntimeBackend {
    match project_type {
        ProjectType::CreateReactApp => RuntimeBackend::Node,
        ProjectType::NextJs => RuntimeBackend::Node,
        ProjectType::PGLiteApp => RuntimeBackend::Pglite,
        ProjectType::Unknown => RuntimeBackend::Node,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selects_pglite_for_pglite_projects() {
        assert_eq!(select_runtime(&ProjectType::PGLiteApp), RuntimeBackend::Pglite);
    }

    #[test]
    fn selects_node_for_frontend_projects() {
        assert_eq!(select_runtime(&ProjectType::CreateReactApp), RuntimeBackend::Node);
        assert_eq!(select_runtime(&ProjectType::NextJs), RuntimeBackend::Node);
    }
}
