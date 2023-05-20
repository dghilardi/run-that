use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use anyhow::bail;
use crate::config::{ScriptBucketDefinition, ScriptSource};
use crate::registry::source::RepoSource;

pub struct ScriptRegistry {
    priority: usize,
    path: PathBuf,
}

impl ScriptRegistry {
    pub fn initialize(conf: &ScriptBucketDefinition) -> anyhow::Result<Self> {
        let path = match &conf.source {
            ScriptSource::Git(git_conf) => git_conf.reify()?,
        };

        Ok(Self {
            priority: conf.priority,
            path,
        })
    }

    pub fn priority(&self) -> usize {
        self.priority
    }

    pub fn has_script(&self, name: &str) -> bool {
        self.script_path(name).is_some()
    }

    fn script_path(&self, name: &str) -> Option<PathBuf> {
        [
            self.path.join(name),
            self.path.join("bin").join(name)
        ]
            .into_iter()
            .filter(|p| p.is_file())
            .next()
    }

    pub fn run_script(&self, name: &str, args: &[String], working_dir: &Path) -> anyhow::Result<()> {
        let Some(path) = self.script_path(name) else {
            bail!("Could not find path for script {name}")
        };

        let out = Command::new(path)
            .args(args)
            .current_dir(working_dir)
            .stdin(Stdio::inherit())
            .stderr(Stdio::inherit())
            .stdout(Stdio::inherit())
            .output()?;

        if out.status.success() {
            Ok(())
        } else {
            bail!("Script exited with error [{}]", out.status)
        }
    }
}