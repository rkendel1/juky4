use std::collections::HashMap;
use std::process::{Child, Command};
use uuid::Uuid;

use crate::kernel::classifier::{classify_repo, ProjectType};
use crate::kernel::execution::session::{ExecutionSession, SessionStatus};
use crate::kernel::ir::compile_ir;
use crate::kernel::runtime::selector::{select_runtime, RuntimeBackend};

/// Manages execution sessions with lifecycle, ports, and process tracking
pub struct ExecutionManager {
    sessions: HashMap<String, ExecutionSession>,
    processes: HashMap<String, Child>,
    next_port: u16,
}

impl Default for ExecutionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ExecutionManager {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            processes: HashMap::new(),
            next_port: 3000,
        }
    }

    /// Allocate the next available port
    fn allocate_port(&mut self) -> u16 {
        let port = self.next_port;
        self.next_port += 1;
        port
    }

    /// Create a new execution session for a repo (does not start it)
    pub fn create_session(&mut self, repo_path: &str) -> String {
        let session_id = Uuid::new_v4().to_string();
        let project_type = classify_repo(repo_path);
        let runtime = select_runtime(&project_type);

        let session = ExecutionSession::new(
            session_id.clone(),
            repo_path.to_string(),
            project_type,
            runtime,
        );

        self.sessions.insert(session_id.clone(), session);
        session_id
    }

    /// Start a session: install dependencies, then run
    pub fn start_session(&mut self, session_id: &str) -> Result<(), String> {
        // Allocate port first (before borrowing session mutably)
        let port = self.allocate_port();

        let session = self
            .sessions
            .get_mut(session_id)
            .ok_or_else(|| "session not found".to_string())?;

        let ir = compile_ir(session.project_type.clone());
        session.port = Some(port);

        // Install phase
        session.transition(SessionStatus::Installing);
        session.log(format!("running: {}", ir.install_cmd));

        let install_status = Command::new("sh")
            .arg("-c")
            .arg(&ir.install_cmd)
            .current_dir(&session.repo_path)
            .status();

        if !install_status.is_ok_and(|s| s.success()) {
            session.transition(SessionStatus::Failed);
            return Err("install failed".into());
        }

        // Determine dev command with port override where applicable
        let dev_cmd = match session.runtime {
            RuntimeBackend::Node => {
                if session.project_type == ProjectType::NextJs {
                    format!("PORT={} {}", port, ir.dev_cmd)
                } else if session.project_type == ProjectType::CreateReactApp {
                    format!("PORT={} {}", port, ir.dev_cmd)
                } else {
                    ir.dev_cmd.clone()
                }
            }
            RuntimeBackend::Pglite => ir.dev_cmd.clone(),
        };

        session.log(format!("starting: {}", dev_cmd));

        let child = Command::new("sh")
            .arg("-c")
            .arg(&dev_cmd)
            .current_dir(&session.repo_path)
            .spawn();

        match child {
            Ok(process) => {
                session.pid = Some(process.id());
                session.transition(SessionStatus::Running);

                // Store process handle for later stop/cleanup
                let sid = session_id.to_string();
                let _ = session; // release mutable borrow
                self.processes.insert(sid, process);
                Ok(())
            }
            Err(e) => {
                session.transition(SessionStatus::Failed);
                Err(format!("spawn failed: {}", e))
            }
        }
    }

    /// Stop a running session
    pub fn stop_session(&mut self, session_id: &str) -> Result<(), String> {
        if let Some(mut child) = self.processes.remove(session_id) {
            let _ = child.kill();
            let _ = child.wait();
        }

        let session = self
            .sessions
            .get_mut(session_id)
            .ok_or_else(|| "session not found".to_string())?;

        session.transition(SessionStatus::Stopped);
        session.pid = None;
        Ok(())
    }

    /// Get session by id
    pub fn get_session(&self, session_id: &str) -> Option<&ExecutionSession> {
        self.sessions.get(session_id)
    }

    /// List all sessions
    pub fn list_sessions(&self) -> Vec<&ExecutionSession> {
        self.sessions.values().collect()
    }

    /// Count running sessions
    pub fn running_count(&self) -> usize {
        self.sessions
            .values()
            .filter(|s| s.status == SessionStatus::Running)
            .count()
    }

    /// Clean up all sessions (stop all processes)
    pub fn cleanup(&mut self) {
        let session_ids: Vec<_> = self.sessions.keys().cloned().collect();
        for id in session_ids {
            let _ = self.stop_session(&id);
        }
    }
}

impl Drop for ExecutionManager {
    fn drop(&mut self) {
        self.cleanup();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_session_with_unique_id() {
        let mut manager = ExecutionManager::new();
        let id1 = manager.create_session("/tmp/fake-repo-1");
        let id2 = manager.create_session("/tmp/fake-repo-2");

        assert_ne!(id1, id2);
        assert!(manager.get_session(&id1).is_some());
        assert!(manager.get_session(&id2).is_some());
    }

    #[test]
    fn allocates_sequential_ports() {
        let mut manager = ExecutionManager::new();
        assert_eq!(manager.allocate_port(), 3000);
        assert_eq!(manager.allocate_port(), 3001);
        assert_eq!(manager.allocate_port(), 3002);
    }

    #[test]
    fn list_sessions_returns_all() {
        let mut manager = ExecutionManager::new();
        manager.create_session("/tmp/a");
        manager.create_session("/tmp/b");
        manager.create_session("/tmp/c");

        assert_eq!(manager.list_sessions().len(), 3);
    }
}
