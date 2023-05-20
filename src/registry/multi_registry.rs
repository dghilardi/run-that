use crate::config::ScriptBucketDefinition;
use crate::registry::script_registry::ScriptRegistry;

pub struct MultiRegistry {
    registries: Vec<ScriptRegistry>,
}

impl MultiRegistry {
    pub fn initialize(conf: Vec<&ScriptBucketDefinition>) -> anyhow::Result<Self> {
        let registries = conf.into_iter()
            .map(ScriptRegistry::initialize)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self { registries })
    }
}