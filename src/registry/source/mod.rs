use std::path::PathBuf;

mod git;
mod cache;

pub trait RepoSource {
    fn reify(&self) -> anyhow::Result<PathBuf>;
}