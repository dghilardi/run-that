use std::path::PathBuf;
use crate::config::{ScriptBucketDefinition, ScriptSource};
use crate::registry::source::RepoSource;

pub struct ScriptRegistry {
    path: PathBuf,
}

impl ScriptRegistry {
    pub fn initialize(conf: &ScriptBucketDefinition) -> anyhow::Result<Self> {
        let path = match &conf.source {
            ScriptSource::Git(git_conf) => git_conf.reify()?,
        };

        Ok(Self {
            path
        })
    }
}