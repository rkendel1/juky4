use std::fs;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectType {
    CreateReactApp,
    NextJs,
    PGLiteApp,
    Unknown,
}

pub fn classify_repo(repo_path: &str) -> ProjectType {
    let path = Path::new(repo_path);
    let pkg_json = fs::read_to_string(path.join("package.json")).unwrap_or_default();

    if pkg_json.contains("\"next\"") {
        return ProjectType::NextJs;
    }

    if pkg_json.contains("\"react-scripts\"") {
        return ProjectType::CreateReactApp;
    }

    if pkg_json.contains("pglite") || pkg_json.contains("@electric-sql") {
        return ProjectType::PGLiteApp;
    }

    ProjectType::Unknown
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn create_temp_repo_with_package_json(contents: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "juky4-test-{}",
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
    fn classifies_next_js_repo() {
        let dir = create_temp_repo_with_package_json(r#"{"dependencies":{"next":"14.0.0"}}"#);
        assert_eq!(classify_repo(dir.to_str().unwrap_or_default()), ProjectType::NextJs);
        fs::remove_dir_all(dir).expect("temp dir should be removed");
    }

    #[test]
    fn classifies_cra_repo() {
        let dir = create_temp_repo_with_package_json(r#"{"dependencies":{"react-scripts":"5.0.1"}}"#);
        assert_eq!(
            classify_repo(dir.to_str().unwrap_or_default()),
            ProjectType::CreateReactApp
        );
        fs::remove_dir_all(dir).expect("temp dir should be removed");
    }

    #[test]
    fn classifies_pglite_repo() {
        let dir = create_temp_repo_with_package_json(r#"{"dependencies":{"pglite":"0.2.0"}}"#);
        assert_eq!(
            classify_repo(dir.to_str().unwrap_or_default()),
            ProjectType::PGLiteApp
        );
        fs::remove_dir_all(dir).expect("temp dir should be removed");
    }
}
