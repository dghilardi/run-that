use std::path::Path;
use anyhow::bail;
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

    pub fn run_script(&self, name: &str, args: &[String], working_dir: &Path) -> anyhow::Result<()> {
        let mut registry = self.registries.iter()
            .filter(|reg| reg.has_script(name))
            .collect::<Vec<_>>();


        registry.sort_by_key(|reg| std::cmp::Reverse(reg.priority()));
        match &registry[..] {
            &[] => bail!("Script {name} was not found in any registry"),
            &[fst, snd] | &[fst, snd, ..] if fst.priority() == snd.priority() => {
                bail!("Script {name} was found in many registries with same priority")
            },
            &[fst] | &[fst, ..] => fst.run_script(name, args, working_dir)?,
        }

        Ok(())
    }
}