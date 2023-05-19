use std::collections::HashMap;
use std::path::Path;
use config::{Config, ConfigBuilder, File, FileFormat};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct RunThatConfig {
    buckets: HashMap<String, Option<ScriptBucketDefinition>>,
}

#[derive(Deserialize)]
pub struct ScriptBucketDefinition {
    #[serde(default)]
    pub priority: usize,
    pub source: ScriptSource,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum ScriptSource {
    Git {
        url: String,
        reference: String,
    }
}

impl RunThatConfig {
    pub fn buckets(&self) -> Vec<&ScriptBucketDefinition> {
        let mut buckets: Vec<_> = self.buckets.iter()
            .flat_map(|(_k, v)| v.as_ref())
            .collect();

        buckets.sort_by_key(|b| b.priority);

        buckets
    }
}

pub fn load_config(working_dir: impl AsRef<Path>) -> anyhow::Result<RunThatConfig> {
    let mut builder = Config::builder();
    let mut dir = working_dir.as_ref();

    loop {
        let config_dir = dir.join(".run-that");
        builder = builder.add_source(File::from(config_dir).format(FileFormat::Toml).required(false));
        let Some(parent) = dir.parent() else {
            break;
        };
        dir = parent;
    }

    let s = builder.build()?;
    Ok(s.try_deserialize()?)
}