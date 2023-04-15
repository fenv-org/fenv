use crate::args;
use anyhow::{ensure, Context as _};
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
    let mut sdks_by_refs = FenvInstallService::list_remote_sdks_by_refs()?;
    let mut merged: Vec<GitRefs> = Vec::new();
    merged.append(&mut sdks_by_tags);
    merged.append(&mut sdks_by_refs);
    Ok(merged)
  }

  fn list_remote_sdks_by_tags() -> Result<Vec<GitRefs>> {
    let command_output = Command::new("bash")
      .arg("-c")
      .arg("git ls-remote --tags https://github.com/flutter/flutter.git")
      .output()
      .context("Failed to fetch remote tags from `https://github.com/flutter/flutter.git`")?;

    ensure!(
      command_output.status.success(),
      "Failed to fetch remote tags from `https://github.com/flutter/flutter.git`"
    );

    println!("list_remote_sdks_by_tags: status={}", command_output.status);
    let git_output = String::from_utf8(command_output.stdout)?;
    Ok(FenvInstallService::parse_git_ls_remote_outputs(&git_output))
  }

  fn list_remote_sdks_by_refs() -> Result<Vec<GitRefs>> {
    let command_output = Command::new("bash")
      .arg("-c")
      .arg("--heads -refs https://github.com/flutter/flutter.git stable beta master")
      .output()?;
    let git_output = String::from_utf8(command_output.stdout)?;
    Ok(FenvInstallService::parse_git_ls_remote_outputs(&git_output))
  }

  fn parse_git_ls_remote_outputs(git_output: &str) -> Vec<GitRefs> {
    // let git_output = String::from_utf8(raw_outputs)?;
    let mut lines = git_output.split("\n");
    lines
      .by_ref()
      .map(|line| GitRefs::parse(line))
      .flatten()
      .collect::<Vec<GitRefs>>()
  }
}

#[derive(Debug)]
pub struct GitRefs {
  pub kind: GitRefsKind,
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
        Some(GitRefs {
          kind,
          sha: String::from(sha),
          short: String::from(short),
          long: String::from(long),
        })
      }
      None => return None,
    }
  }
}
