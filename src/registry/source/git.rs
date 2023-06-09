use std::fs;
use std::ops::Add;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use anyhow::bail;
use chrono::{Duration, Utc};

use crate::config::GitSourceConfig;
use crate::registry::source::cache::CacheMeta;
use crate::registry::source::RepoSource;

impl RepoSource for GitSourceConfig {
    fn reify(&self) -> anyhow::Result<PathBuf> {
        let cache_path = self.build_cache_path();
        fs::create_dir_all(&cache_path)?;

        let cache_meta_path = cache_path.join("meta.toml");
        let cache_meta: Option<CacheMeta> = if cache_meta_path.is_file() {
            Some(toml::from_str(&fs::read_to_string(&cache_meta_path)?)?)
        } else {
            None
        };

        match cache_meta {
            None => {
                self.update_head(&cache_path)?;
                let cache_meta_str = toml::to_string_pretty(&CacheMeta { last_update: Utc::now() })?;
                fs::write(&cache_meta_path, cache_meta_str)?;
            }
            Some(mut meta) if meta.last_update.add(Duration::days(1)) < Utc::now() || !cache_path.join(&self.reference).is_dir() => {
                self.update_head(&cache_path)?;
                meta.last_update = Utc::now();
                let cache_meta_str = toml::to_string_pretty(&meta)?;
                fs::write(&cache_meta_path, cache_meta_str)?;
            }
            Some(_) => {
                eprintln!("Skip cache update for {}", self.repo_name())
            }
        }
        let dir = self.verify_ref_dir(&cache_path)?;

        Ok(dir)
    }
}

impl GitSourceConfig {
    fn build_cache_path(&self) -> PathBuf {
        let repo_name = self.repo_name();
        let repo_unique_name = format!("{repo_name}-{}", self.repo_hash());

        dirs::cache_dir()
            .unwrap_or(PathBuf::from("/tmp"))
            .join("run-that")
            .join("git")
            .join(repo_unique_name)
    }

    fn repo_name(&self) -> &str {
        let last_chunk = self.url.split('/').last()
            .expect("Cannot extract last chunk from git repo url");

        last_chunk
            .strip_suffix(".git")
            .unwrap_or(last_chunk)
    }

    fn repo_hash(&self) -> String {
        sha256::digest(self.url.as_bytes())
    }

    fn update_head(&self, cache_dir: &Path) -> anyhow::Result<()> {
        let head_dir = cache_dir.join("HEAD");
        if head_dir.is_dir() {
            let out = Command::new("git")
                .arg("pull")
                .arg("--ff-only")
                .current_dir(head_dir)
                .stdin(Stdio::inherit())
                .stderr(Stdio::inherit())
                .stdout(Stdio::null())
                .output()?;

            if out.status.success() {
                Ok(())
            } else {
                bail!("Error updating repo HEAD [{}]", out.status)
            }
        } else {
            let out = Command::new("git")
                .arg("clone")
                .arg(&self.url)
                .arg(head_dir)
                .stdin(Stdio::inherit())
                .stderr(Stdio::inherit())
                .stdout(Stdio::null())
                .output()?;

            if out.status.success() {
                Ok(())
            } else {
                bail!("Error cloning repo HEAD [{}]", out.status)
            }
        }
    }

    fn verify_ref_dir(&self, cache_dir: &Path) -> anyhow::Result<PathBuf> {
        let ref_dir = cache_dir.join(&self.reference);
        if !ref_dir.is_dir() {
            let out = Command::new("git")
                .arg("worktree")
                .arg("add")
                .arg(&ref_dir)
                .arg(&self.reference)
                .current_dir(cache_dir.join("HEAD"))
                .stdin(Stdio::inherit())
                .stderr(Stdio::inherit())
                .stdout(Stdio::null())
                .output()?;

            if out.status.success() {
                self.check_repo_updated(&ref_dir)?;

                Ok(ref_dir)
            } else {
                bail!("Error creating worktree for {} [{}]", self.reference, out.status)
            }
        } else {
            self.check_repo_updated(&ref_dir)?;
            Ok(ref_dir)
        }
    }

    fn check_repo_updated(&self, dir: &Path) -> anyhow::Result<()> {
        let distance = self.get_commit_distance(dir, "HEAD", "origin/HEAD")?;
        if distance > 0 {
            eprintln!("WARNING: origin/HEAD is {distance} commit ahead from {} in repo {}", self.reference, self.url);
        }
        Ok(())
    }

    fn get_commit_distance(&self, dir: &Path, ref_a: &str, ref_b: &str) -> anyhow::Result<usize> {
        let hash_a = self.get_commit_hash(dir, ref_a)?;
        let hash_b = self.get_commit_hash(dir, ref_b)?;

        if hash_a.eq(&hash_b) {
            return Ok(0);
        }

        let out = Command::new("git")
            .arg("rev-list")
            .arg(format!("{hash_a}...{hash_b}"))
            .current_dir(dir)
            .output()?;

        if out.status.success() {
            let rev_list_out = String::from_utf8(out.stdout)?;
            Ok(
                rev_list_out
                    .split('\n')
                    .filter(|s| !s.is_empty())
                    .count()
            )
        } else {
            bail!("Error getting distance ({ref_a} .. {ref_b}) for {} [{}]", self.reference, out.status)
        }
    }

    fn get_commit_hash(&self, dir: &Path, commit_ref: &str) -> anyhow::Result<String> {
        let out = Command::new("git")
            .arg("rev-parse")
            .arg(commit_ref)
            .current_dir(dir)
            .output()?;

        if out.status.success() {
            Ok(String::from_utf8(out.stdout)?.trim().to_string())
        } else {
            bail!("Error getting {commit_ref} for {} [{}]", self.reference, out.status)
        }
    }
}