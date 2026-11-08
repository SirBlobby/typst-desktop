use std::collections::HashSet;
use std::sync::{LazyLock, Mutex};

static RUNNING: LazyLock<Mutex<HashSet<String>>> = LazyLock::new(Default::default);

pub struct ProjectLock {
    project_path: String,
}

impl Drop for ProjectLock {
    fn drop(&mut self) {
        if let Ok(mut running) = RUNNING.lock() {
            running.remove(&self.project_path);
        }
    }
}

pub fn lock_project(project_path: &str) -> Result<ProjectLock, String> {
    let mut running = RUNNING.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    if !running.insert(project_path.to_string()) {
        return Err("A GitHub sync is already running for this project. Try again in a moment.".to_string());
    }
    Ok(ProjectLock { project_path: project_path.to_string() })
}
