use std::collections::HashMap;
use std::path::Path;
use std::str::FromStr;
use anyhow::{anyhow, bail};
use config::{Config, File, FileFormat};
use serde::de::Error;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct RunThatConfig {
    buckets: HashMap<String, Option<ScriptBucketDefinition>>,
}

#[derive(Deserialize)]
#[serde(from = "ScriptBucketDefinitionDes")]
pub struct ScriptBucketDefinition {
    pub priority: usize,
    pub source: ScriptSource,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum ScriptSource {
    Git(GitSourceConfig)
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum ScriptBucketDefinitionDes {
    #[serde(deserialize_with = "deserialize_source_from_str")]
    FromStr(InnerScriptBucketDefinition),
    FromObj(InnerScriptBucketDefinition),
}

#[derive(Deserialize)]
pub struct InnerScriptBucketDefinition {
    #[serde(default)]
    pub priority: usize,
    pub source: ScriptSource,
}

impl From<ScriptBucketDefinitionDes> for ScriptBucketDefinition {
    fn from(value: ScriptBucketDefinitionDes) -> Self {
        match value {
            ScriptBucketDefinitionDes::FromStr(InnerScriptBucketDefinition { priority, source }) => ScriptBucketDefinition { priority, source },
            ScriptBucketDefinitionDes::FromObj(InnerScriptBucketDefinition { priority, source }) => ScriptBucketDefinition { priority, source },
        }
    }
}

fn deserialize_source_from_str<'de, D>(deserializer: D) -> Result<InnerScriptBucketDefinition, D::Error>
    where
        D: serde::Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    InnerScriptBucketDefinition::from_str(&value)
        .map_err(|err| D::Error::custom(err.to_string()))
}

impl FromStr for InnerScriptBucketDefinition {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let Some(sep_idx) = value.find(':') else {
            return Err(anyhow!("could not find ':' separator"));
        };

        let source = match &value[0..sep_idx] {
            "git" => GitSourceConfig::from_str(&value[sep_idx+1..])
                .map(ScriptSource::Git)?,
            o => bail!("{o} source type is not currently handled")
        };

        Ok(InnerScriptBucketDefinition {
            priority: Default::default(),
            source,
        })
    }
}

#[derive(Deserialize)]
pub struct GitSourceConfig {
    pub url: String,
    pub reference: String,
}

impl FromStr for GitSourceConfig {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let hash_idx = s.find('#')
            .ok_or_else(|| anyhow!("Could not find # in git url"))?;

        Ok(Self {
            url: String::from(&s[0..hash_idx]),
            reference: String::from(&s[hash_idx+1..]),
        })
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

    let mut paths = vec![];

    loop {
        let config_dir = dir.join(".run-that");
        paths.push(File::from(config_dir).format(FileFormat::Toml).required(false));
        let Some(parent) = dir.parent() else {
            break;
        };
        dir = parent;
    }

    for path in paths.into_iter().rev() {
        builder = builder.add_source(path);
    }

    builder = builder.add_source(config::Environment::with_prefix("RT").separator("_").prefix_separator("_"));

    let s = builder.build()?;
    Ok(s.try_deserialize()?)
}

#[cfg(test)]
mod test {
    use crate::config::{RunThatConfig, ScriptBucketDefinition, ScriptSource};

    #[test]
    fn asd() {
        let input = r###"
        buckets.inline = "git:git@gitlab.com:dghilardi/demo-1.git#12345"
        [buckets.expanded]
        source.type = "Git"
        source.url = "git@gitlab.com:dghilardi/demo-2.git"
        source.reference = "54321"
        "###;

        let parsed = toml::from_str::<RunThatConfig>(input).expect("Error parsing config");

        let Some(Some(ScriptBucketDefinition { source: ScriptSource::Git(inline_conf), .. })) = parsed.buckets.get("inline") else {
            panic!("Cannot find inline source");
        };

        assert_eq!("git@gitlab.com:dghilardi/demo-1.git", inline_conf.url);
        assert_eq!("12345", inline_conf.reference);

        let Some(Some(ScriptBucketDefinition { source: ScriptSource::Git(expanded_conf), .. })) = parsed.buckets.get("expanded") else {
            panic!("Cannot find expanded source");
        };

        assert_eq!("git@gitlab.com:dghilardi/demo-2.git", expanded_conf.url);
        assert_eq!("54321", expanded_conf.reference);
    }
}