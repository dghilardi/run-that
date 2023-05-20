use std::path::PathBuf;

mod git;

pub trait RepoSource {
    fn reify(&self) -> anyhow::Result<PathBuf>;
}