#![feature(default_field_values)]
mod check;
mod frontmatter;
mod git;
mod install;
mod lock;

use std::path::Path;

use clap::Parser;
use miette::Result;

/// Cargo-style version requirements, lockfile and install for agent skills.
#[derive(Parser)]
#[command(version)]
enum Cli {
	/// Bring skargo.lock in the current directory in line with skargo.toml, keeping pins it still allows.
	Lock,
	/// `lock`, then link every locked skill into ~/.claude/skills and ~/.agents/skills.
	Install,
	/// From inside a skills repo: validate every tracked SKILL.md, regenerate .claude-plugin/marketplace.json plugins, print the tags still to release.
	Check,
}

fn main() -> Result<()> {
	match Cli::parse() {
		Cli::Lock => lock::sync(Path::new(".")).map(drop),
		Cli::Install => install::install(&lock::sync(Path::new("."))?),
		Cli::Check => {
			for tag in check::check()? {
				println!("{tag}");
			}
			Ok(())
		}
	}
}
