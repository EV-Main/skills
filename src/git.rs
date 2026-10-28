use std::{collections::BTreeMap, path::Path, process::Command};

use miette::{IntoDiagnostic, Result, WrapErr, bail};
use tempfile::TempDir;

/// Skill names can't contain `--` (agentskills spec), so the first occurrence splits unambiguously.
pub(crate) const TAG_SEPARATOR: &str = "--v";

pub(crate) fn bytes(dir: &Path, args: &[&str]) -> Result<Vec<u8>> {
	let out = command(dir, args).output().into_diagnostic().wrap_err("spawning git")?;
	if !out.status.success() {
		bail!("`git {}` in {} failed: {}", args.join(" "), dir.display(), String::from_utf8_lossy(&out.stderr).trim());
	}
	Ok(out.stdout)
}

pub(crate) fn text(dir: &Path, args: &[&str]) -> Result<String> {
	String::from_utf8(bytes(dir, args)?)
		.into_diagnostic()
		.wrap_err_with(|| format!("`git {}` printed non-utf8", args.join(" ")))
}

/// For commands whose exit code 1 is an answer (`diff --quiet`), not a failure.
pub(crate) fn holds(dir: &Path, args: &[&str]) -> Result<bool> {
	let out = command(dir, args).output().into_diagnostic().wrap_err("spawning git")?;
	match out.status.code() {
		Some(0) => Ok(true),
		Some(1) => Ok(false),
		_ => bail!("`git {}` in {} failed: {}", args.join(" "), dir.display(), String::from_utf8_lossy(&out.stderr).trim()),
	}
}

fn command(dir: &Path, args: &[&str]) -> Command {
	let mut c = Command::new("git");
	c.arg("-C").arg(dir).args(args);
	c
}

pub(crate) fn tag(name: &str, version: &semver::Version) -> String {
	format!("{name}{TAG_SEPARATOR}{version}")
}

/// `<name>--v<raw version>` → commit, from tag refs of a remote. Annotated tags resolve to the commit they point at.
pub(crate) fn remote_releases(url: &str) -> Result<BTreeMap<String, BTreeMap<String, String>>> {
	let listing = text(Path::new("."), &["ls-remote", "--tags", url])?;
	let mut direct = BTreeMap::new();
	let mut peeled = BTreeMap::new();
	for line in listing.lines() {
		let (commit, refname) = line.split_once('\t').ok_or_else(|| miette::miette!("unexpected ls-remote line: {line}"))?;
		let tag = refname
			.strip_prefix("refs/tags/")
			.ok_or_else(|| miette::miette!("ls-remote --tags listed a non-tag ref: {refname}"))?;
		match tag.strip_suffix("^{}") {
			Some(tag) => peeled.insert(tag.to_owned(), commit.to_owned()),
			None => direct.insert(tag.to_owned(), commit.to_owned()),
		};
	}
	direct.extend(peeled);
	let mut releases: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
	for (tag, commit) in direct {
		if let Some((name, version)) = tag.split_once(TAG_SEPARATOR) {
			releases.entry(name.to_owned()).or_default().insert(version.to_owned(), commit);
		}
	}
	Ok(releases)
}

/// A throwaway bare repo holding one tag, fetched at depth 1.
pub(crate) struct Fetched {
	repo: TempDir,
	pub(crate) commit: String,
}

impl Fetched {
	/// By tag, not by sha: fetching an arbitrary sha needs server opt-in on some transports.
	pub(crate) fn tag(url: &str, tag: &str) -> Result<Self> {
		let repo = tempfile::tempdir().into_diagnostic()?;
		bytes(repo.path(), &["init", "-q", "--bare"])?;
		bytes(repo.path(), &["fetch", "-q", "--depth", "1", url, &format!("refs/tags/{tag}")])?;
		let commit = text(repo.path(), &["rev-parse", "FETCH_HEAD^{commit}"])?.trim().to_owned();
		Ok(Self { repo, commit })
	}

	pub(crate) fn path(&self) -> &Path {
		self.repo.path()
	}
}
