#![feature(default_field_values)]
mod check;
mod frontmatter;
mod git;
mod install;
mod lock;

use std::path::Path;

use clap::Parser;
use lock::Refresh;
use miette::Result;

/// Cargo-style version requirements, lockfile and install for agent skills.
#[derive(Parser)]
#[command(version)]
enum Cli {
	/// Bring skargo.lock in the current directory in line with skargo.toml, keeping pins it still allows.
	Lock,
	/// `lock`, re-resolving the named skills (all, if none are named) instead of keeping their pins.
	Update { names: Vec<String> },
	/// `lock`, then link every locked skill into ~/.claude/skills and ~/.agents/skills.
	Install,
	/// From inside a skills repo: validate every tracked SKILL.md, sync its skargo.lock, regenerate .claude-plugin/marketplace.json plugins, print the tags still to release.
	Check,
	/// `check`, then tag HEAD with every tag it prints and push them to origin.
	Publish,
}

fn main() -> Result<()> {
	match Cli::parse() {
		Cli::Lock => lock::sync(Path::new("."), Refresh::None).map(drop),
		Cli::Update { names } => lock::sync(Path::new("."), if names.is_empty() { Refresh::All } else { Refresh::Only(&names) }).map(drop),
		Cli::Install => install::install(&lock::sync(Path::new("."), Refresh::None)?),
		Cli::Check => {
			for tag in check::check()? {
				println!("{tag}");
			}
			Ok(())
		}
		Cli::Publish => check::publish(),
	}
}
