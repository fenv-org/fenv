use crate::args;
use anyhow::{bail, Context as _};
use anyhow::{Ok, Result};
use lazy_static::lazy_static;
use regex::Regex;
use std::process::Command;

pub struct FenvInstallService {
  pub args: args::FenvInstallArgs,
}

impl FenvInstallService {
  pub fn from(args: args::FenvInstallArgs) -> FenvInstallService {
    return FenvInstallService { args };
  }

  pub fn execute(&self) -> Result<()> {
    if self.args.list {
      let sdks = FenvInstallService::list_remote_sdks()?;
      for sdk in sdks {
        println!("{:?}", sdk);
      }
    }
    Ok(())
  }

  pub fn list_remote_sdks() -> Result<Vec<GitRefs>> {
    let mut sdks_by_tags = FenvInstallService::list_remote_sdks_by_tags()?;
    let mut sdks_by_refs = FenvInstallService::list_remote_sdks_by_branches()?;
    let mut merged: Vec<GitRefs> = Vec::new();
    merged.append(&mut sdks_by_tags);
    merged.append(&mut sdks_by_refs);
    Ok(merged)
  }

  fn list_remote_sdks_by_tags() -> Result<Vec<GitRefs>> {
    lazy_static! {
      static ref ACCEPTABLE_TAGS_PATTERN: Regex =
        Regex::new(r"^v?(\d+\.\d+\.\d+(?:(?:\+|-)hotfix\.\d+)?)$").unwrap();
    }

    const ERROR_MESSAGE: &str =
      "Failed to fetch remote tags from `https://github.com/flutter/flutter.git`";

    let command_output = Command::new("git")
      .arg("ls-remote")
      .arg("--tags")
      .arg("https://github.com/flutter/flutter.git")
      .output()
      .context(ERROR_MESSAGE)?;
    if !command_output.status.success() {
      bail!("{}: {}", ERROR_MESSAGE, command_output.status);
    }
    let git_output = String::from_utf8(command_output.stdout)?;
    let mut lines = git_output.split("\n");
    let git_refs = lines
      .by_ref()
      .map(|line| GitRefs::parse(line))
      .flatten()
      .filter(|git_refs| ACCEPTABLE_TAGS_PATTERN.is_match(&git_refs.short))
      .collect::<Vec<GitRefs>>();
    Ok(git_refs)
  }

  fn list_remote_sdks_by_branches() -> Result<Vec<GitRefs>> {
    const ERROR_MESSAGE: &str =
      "Failed to fetch remote branches from `https://github.com/flutter/flutter.git`";

    let command_output = Command::new("git")
      .arg("ls-remote")
      .args(["--heads", "--refs"])
      .arg("https://github.com/flutter/flutter.git")
      .args(["stable", "beta", "master"])
      .output()
      .context(ERROR_MESSAGE)?;
    if !command_output.status.success() {
      bail!("{}: {}", ERROR_MESSAGE, command_output.status);
    }
    let git_output = String::from_utf8(command_output.stdout)?;
    let mut lines = git_output.split("\n");
    let git_refs = lines
      .by_ref()
      .map(|line| GitRefs::parse(line))
      .flatten()
      .collect::<Vec<GitRefs>>();
    Ok(git_refs)
  }
}

#[derive(Debug)]
pub struct GitRefs {
  pub kind: GitRefsKind,
  pub name: String,
  pub sha: String,
  pub short: String,
  pub long: String,
}

#[derive(Debug)]
pub enum GitRefsKind {
  Tag,
  Head,
}

impl GitRefs {
  fn parse(line: &str) -> Option<GitRefs> {
    lazy_static! {
      static ref PATTERN: Regex =
        Regex::new(r"^([a-z0-9]+)\s+(refs/((?:tags)|(?:heads))/(\S+))").unwrap();
    }
    match PATTERN.captures(line) {
      Some(capture) => {
        let sha = capture.get(1).map_or("", |m| m.as_str());
        let long = capture.get(2).map_or("", |m| m.as_str());
        let tags_or_heads = capture.get(3).map_or("", |m| m.as_str());
        let short = capture.get(4).map_or("", |m| m.as_str());
        let kind = match tags_or_heads {
          "tags" => GitRefsKind::Tag,
          "heads" => GitRefsKind::Head,
          _ => return None,
        };
        let name = match kind {
          GitRefsKind::Tag => {
            if short.starts_with("v") {
              String::from(&short[1..])
            } else {
              String::from(&short[..])
            }
          }
          GitRefsKind::Head => String::from(&short[..]),
        };
        Some(GitRefs {
          kind,
          sha: String::from(sha),
          short: String::from(short),
          long: String::from(long),
          name,
        })
      }
      None => return None,
    }
  }
}
