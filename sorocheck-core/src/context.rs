use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Context {
    pub project_root: PathBuf,
}

impl Context {
    /// Build a context rooted at the given project path.
    pub fn new(project_root: &Path) -> Self {
        Self {
            project_root: project_root.to_path_buf(),
        }
    }

    /// The root directory of the project being checked.
    pub fn project_root(&self) -> &Path {
        &self.project_root
    }
}