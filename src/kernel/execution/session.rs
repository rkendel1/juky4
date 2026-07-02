use crate::kernel::classifier::ProjectType;
use crate::kernel::runtime::selector::RuntimeBackend;

/// Lifecycle state of an execution session
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionStatus {
    Created,
    Installing,
    Running,
    Failed,
    Stopped,
}

/// Managed execution session with lifecycle tracking
#[derive(Debug, Clone)]
pub struct ExecutionSession {
    pub session_id: String,
    pub repo_path: String,
    pub project_type: ProjectType,
    pub runtime: RuntimeBackend,
    pub status: SessionStatus,
    pub pid: Option<u32>,
    pub port: Option<u16>,
    pub logs: Vec<String>,
}

impl ExecutionSession {
    pub fn new(
        session_id: String,
        repo_path: String,
        project_type: ProjectType,
        runtime: RuntimeBackend,
    ) -> Self {
        Self {
            session_id,
            repo_path,
            project_type,
            runtime,
            status: SessionStatus::Created,
            pid: None,
            port: None,
            logs: Vec::new(),
        }
    }

    pub fn log(&mut self, message: impl Into<String>) {
        self.logs.push(message.into());
    }

    pub fn transition(&mut self, new_status: SessionStatus) {
        self.log(format!(
            "status transition: {:?} -> {:?}",
            self.status, new_status
        ));
        self.status = new_status;
    }

    pub fn is_running(&self) -> bool {
        self.status == SessionStatus::Running
    }

    pub fn url(&self) -> Option<String> {
        match (&self.runtime, self.port) {
            (RuntimeBackend::Node, Some(port)) => Some(format!("http://localhost:{}", port)),
            (RuntimeBackend::Pglite, _) => Some("pglite://local-db".into()),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_starts_in_created_status() {
        let session = ExecutionSession::new(
            "test-1".into(),
            "/tmp/repo".into(),
            ProjectType::NextJs,
            RuntimeBackend::Node,
        );
        assert_eq!(session.status, SessionStatus::Created);
        assert!(session.pid.is_none());
        assert!(session.port.is_none());
    }

    #[test]
    fn session_transitions_log_status_changes() {
        let mut session = ExecutionSession::new(
            "test-2".into(),
            "/tmp/repo".into(),
            ProjectType::CreateReactApp,
            RuntimeBackend::Node,
        );
        session.transition(SessionStatus::Installing);
        session.transition(SessionStatus::Running);

        assert_eq!(session.status, SessionStatus::Running);
        assert!(session.logs.iter().any(|l| l.contains("Installing")));
        assert!(session.logs.iter().any(|l| l.contains("Running")));
    }

    #[test]
    fn url_returns_correct_endpoint() {
        let mut session = ExecutionSession::new(
            "test-3".into(),
            "/tmp/repo".into(),
            ProjectType::NextJs,
            RuntimeBackend::Node,
        );
        session.port = Some(3001);
        assert_eq!(session.url(), Some("http://localhost:3001".into()));

        let pglite_session = ExecutionSession::new(
            "test-4".into(),
            "/tmp/repo".into(),
            ProjectType::PGLiteApp,
            RuntimeBackend::Pglite,
        );
        assert_eq!(pglite_session.url(), Some("pglite://local-db".into()));
    }
}
