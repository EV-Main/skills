use std::{
	collections::HashSet,
	env, fs,
	os::unix::fs::symlink,
	path::{Path, PathBuf},
};

use miette::{IntoDiagnostic, Result, WrapErr, bail, miette};

use crate::{git, lock::Locked};

/// `~/.claude/skills`: Claude Code (and DeepSeek through it), opencode. `~/.agents/skills`: Codex, Kimi Code, opencode, gh skill.
const AGENT_DIRS: [&str; 2] = [".claude/skills", ".agents/skills"];

pub(crate) fn install(locked: &[Locked]) -> Result<()> {
	let home = PathBuf::from(env::var_os("HOME").ok_or_else(|| miette!("$HOME is unset"))?);
	let store = match env::var_os("XDG_DATA_HOME") {
		Some(data) => PathBuf::from(data),
		None => home.join(".local/share"),
	}
	.join("skargo");
	let entry = |l: &Locked| store.join(format!("{}-{}", l.commit, l.name));
	let ours = |link: &Path| fs::read_link(link).is_ok_and(|target| target.starts_with(&store));
	let targets: Vec<PathBuf> = AGENT_DIRS.iter().map(|d| home.join(d)).collect();

	let foreign: Vec<String> = targets
		.iter()
		.flat_map(|t| locked.iter().map(move |l| t.join(&l.name)))
		.filter(|link| link.symlink_metadata().is_ok() && !ours(link))
		.map(|link| link.display().to_string())
		.collect();
	if !foreign.is_empty() {
		bail!("not installed by skargo, left untouched: {}", foreign.join(", "));
	}

	fs::create_dir_all(&store).into_diagnostic().wrap_err_with(|| format!("creating {}", store.display()))?;
	for l in locked {
		let dest = entry(l);
		if !dest.exists() {
			materialize(l, &store, &dest)?;
		}
	}

	for target in &targets {
		fs::create_dir_all(target).into_diagnostic().wrap_err_with(|| format!("creating {}", target.display()))?;
		for l in locked {
			let link = target.join(&l.name);
			if fs::read_link(&link).is_ok_and(|t| t == entry(l)) {
				continue;
			}
			if link.symlink_metadata().is_ok() {
				fs::remove_file(&link).into_diagnostic().wrap_err_with(|| format!("replacing {}", link.display()))?;
			}
			symlink(entry(l), &link).into_diagnostic().wrap_err_with(|| format!("linking {}", link.display()))?;
		}
		let wanted: HashSet<&str> = locked.iter().map(|l| l.name.as_str()).collect();
		for existing in fs::read_dir(target).into_diagnostic()? {
			let path = existing.into_diagnostic()?.path();
			let name = path.file_name().expect("read_dir entries have names").to_string_lossy();
			if !wanted.contains(name.as_ref()) && ours(&path) {
				fs::remove_file(&path).into_diagnostic().wrap_err_with(|| format!("removing {}", path.display()))?;
			}
		}
	}

	let referenced: HashSet<PathBuf> = locked.iter().map(entry).collect();
	for existing in fs::read_dir(&store).into_diagnostic()? {
		let path = existing.into_diagnostic()?.path();
		if !referenced.contains(&path) {
			fs::remove_dir_all(&path).into_diagnostic().wrap_err_with(|| format!("removing {}", path.display()))?;
		}
	}
	Ok(())
}

/// Unpacks next to `dest` and renames, so a present `dest` is always complete.
fn materialize(l: &Locked, store: &Path, dest: &Path) -> Result<()> {
	let fetched = git::Fetched::tag(&l.git, &git::tag(&l.name, &l.version))?;
	if fetched.commit != l.commit {
		bail!(
			"tag {} in {} now points at {}, locked at {}; released tags are immutable",
			git::tag(&l.name, &l.version),
			l.git,
			fetched.commit,
			l.commit
		);
	}
	let archive = git::bytes(fetched.path(), &["archive", "--format=tar", &format!("{}:{}", l.commit, l.path)])?;
	let staging = tempfile::tempdir_in(store).into_diagnostic()?;
	tar::Archive::new(archive.as_slice())
		.unpack(staging.path())
		.into_diagnostic()
		.wrap_err_with(|| format!("unpacking {}", l.name))?;
	fs::rename(staging.keep(), dest).into_diagnostic().wrap_err_with(|| format!("moving {} into the store", l.name))
}
