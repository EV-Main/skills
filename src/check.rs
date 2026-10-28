use std::{
	collections::{BTreeMap, BTreeSet},
	fs,
	path::{Path, PathBuf},
};

use miette::{Diagnostic, IntoDiagnostic, Result, WrapErr};
use pulldown_cmark::{Event, Parser, Tag};
use semver::Version;
use serde_json::{Value, json};

use crate::{frontmatter, git};

const MARKETPLACE: &str = ".claude-plugin/marketplace.json";

#[derive(Debug, Diagnostic, thiserror::Error)]
enum Violation {
	#[error("{skill_md}: {message}")]
	Frontmatter { skill_md: String, message: String },
	#[error("{skill_md} sits inside skill {outer}/; installers scanning recursively would publish it as a second skill")]
	Nested { skill_md: String, outer: String },
	#[error("{file} is a symlink; a skill installs as its own directory, so it can't point anywhere stable")]
	Symlink { file: String },
	#[error("{file}: link {link:?} {problem}")]
	Link { file: String, link: String, problem: &'static str },
	#[error("{0:?} is the frontmatter name of several skills: {1}")]
	DuplicateName(String, String),
	#[error("{dir}: content differs from released {tag} without a version bump")]
	ChangedWithoutBump { dir: String, tag: String },
	#[error("{dir}: version {version} must exceed latest released {latest}")]
	NotAboveLatest { dir: String, version: Version, latest: Version },
	#[error("tag {0} is not <name>--v<semver>")]
	MalformedTag(String),
	#[error("{0} was stale and has been rewritten; stage it")]
	StaleMarketplace(String),
}

struct Skill {
	/// Relative to the repo root, `""` for a skill at the root.
	dir: String,
	name: String,
	description: String,
	version: Version,
}

/// Validates every tracked skill of the repo around the cwd; returns the tags its versions still need.
pub(crate) fn check() -> Result<Vec<String>> {
	let root = PathBuf::from(git::text(Path::new("."), &["rev-parse", "--show-toplevel"])?.trim());
	let mut violations = Vec::new();

	let mut tracked = BTreeSet::new();
	let mut symlinks = BTreeSet::new();
	for entry in git::text(&root, &["ls-files", "-s", "-z"])?.split_terminator('\0') {
		let (meta, path) = entry.split_once('\t').expect("`ls-files -s` is `<mode> <object> <stage>\\t<path>`");
		if meta.starts_with("120000") {
			symlinks.insert(path.to_owned());
		}
		tracked.insert(path.to_owned());
	}

	let skill_dirs: Vec<String> = tracked
		.iter()
		.filter_map(|f| {
			if f == "SKILL.md" {
				Some(String::new())
			} else {
				f.strip_suffix("/SKILL.md").map(str::to_owned)
			}
		})
		.collect();
	let inside = |path: &str, dir: &str| dir.is_empty() || path == dir || path.starts_with(&format!("{dir}/"));

	let mut skills = Vec::new();
	for dir in &skill_dirs {
		let skill_md = join(dir, "SKILL.md");
		if let Some(outer) = skill_dirs.iter().find(|outer| *outer != dir && inside(dir, outer)) {
			violations.push(Violation::Nested {
				skill_md: skill_md.clone(),
				outer: outer.clone(),
			});
		}
		let files: Vec<&String> = tracked.iter().filter(|f| inside(f, dir)).collect();
		for file in &files {
			if symlinks.contains(*file) {
				violations.push(Violation::Symlink { file: (*file).clone() });
			} else if file.ends_with(".md") {
				let content = fs::read_to_string(root.join(file)).into_diagnostic().wrap_err_with(|| format!("reading {file}"))?;
				violations.extend(link_violations(file, &content, dir, &tracked));
			}
		}
		let content = fs::read_to_string(root.join(&skill_md)).into_diagnostic().wrap_err_with(|| format!("reading {skill_md}"))?;
		match frontmatter::parse(&content) {
			Ok(fm) => skills.push(Skill {
				dir: dir.clone(),
				name: fm.name,
				description: fm.description,
				version: fm.version,
			}),
			Err(messages) => violations.extend(messages.into_iter().map(|message| Violation::Frontmatter {
				skill_md: skill_md.clone(),
				message,
			})),
		}
	}

	let mut by_name: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
	for s in &skills {
		by_name.entry(&s.name).or_default().push(&s.dir);
	}
	for (name, dirs) in by_name.iter().filter(|(_, dirs)| dirs.len() > 1) {
		violations.push(Violation::DuplicateName(name.to_string(), dirs.join(", ")));
	}

	let mut released: BTreeMap<String, Vec<Version>> = BTreeMap::new();
	for tag in git::text(&root, &["tag", "--list", &format!("*{}*", git::TAG_SEPARATOR)])?.lines() {
		let (name, raw) = tag.split_once(git::TAG_SEPARATOR).expect("listed by this pattern");
		match Version::parse(raw) {
			Ok(v) => released.entry(name.to_owned()).or_default().push(v),
			Err(_) => violations.push(Violation::MalformedTag(tag.to_owned())),
		}
	}
	let mut pending = Vec::new();
	for s in &skills {
		let tag = git::tag(&s.name, &s.version);
		let versions = released.get(&s.name).map(Vec::as_slice).unwrap_or_default();
		if versions.contains(&s.version) {
			let pathspec = if s.dir.is_empty() { "." } else { &s.dir };
			if !git::holds(&root, &["diff", "--quiet", &tag, "--", pathspec])? {
				violations.push(Violation::ChangedWithoutBump { dir: s.dir.clone(), tag });
			}
		} else if let Some(latest) = versions.iter().max().filter(|latest| **latest >= s.version) {
			violations.push(Violation::NotAboveLatest {
				dir: s.dir.clone(),
				version: s.version.clone(),
				latest: latest.clone(),
			});
		} else {
			pending.push(tag);
		}
	}

	let marketplace = root.join(MARKETPLACE);
	if marketplace.exists() {
		let current = fs::read_to_string(&marketplace).into_diagnostic().wrap_err_with(|| format!("reading {MARKETPLACE}"))?;
		let mut manifest: Value = serde_json::from_str(&current).into_diagnostic().wrap_err_with(|| format!("parsing {MARKETPLACE}"))?;
		let Some(top) = manifest.as_object_mut() else {
			miette::bail!("{MARKETPLACE} is not a JSON object");
		};
		skills.sort_by(|a, b| a.name.cmp(&b.name));
		let plugins = skills
			.iter()
			.map(|s| {
				json!({
					"name": s.name,
					"source": if s.dir.is_empty() { "./".to_owned() } else { format!("./{}", s.dir) },
					"description": s.description,
					"version": s.version.to_string(),
				})
			})
			.collect();
		top.insert("plugins".to_owned(), Value::Array(plugins));
		let rendered = format!("{}\n", serde_json::to_string_pretty(&manifest).expect("a Value always serializes"));
		if rendered != current {
			fs::write(&marketplace, rendered).into_diagnostic().wrap_err_with(|| format!("writing {MARKETPLACE}"))?;
			violations.push(Violation::StaleMarketplace(MARKETPLACE.to_owned()));
		}
	}

	if !violations.is_empty() {
		let count = violations.len();
		for v in violations {
			eprintln!("{:?}", miette::Report::new(v));
		}
		miette::bail!("{count} violation(s)");
	}
	Ok(pending)
}

fn join(dir: &str, file: &str) -> String {
	if dir.is_empty() { file.to_owned() } else { format!("{dir}/{file}") }
}

/// A relative link must land on a tracked file or directory inside the skill, since only the skill's directory gets installed.
fn link_violations(file: &str, content: &str, skill_dir: &str, tracked: &BTreeSet<String>) -> Vec<Violation> {
	let base = file.rsplit_once('/').map_or("", |(dir, _)| dir);
	let mut violations = Vec::new();
	for event in Parser::new(content) {
		let (Event::Start(Tag::Link { dest_url, .. }) | Event::Start(Tag::Image { dest_url, .. })) = event else {
			continue;
		};
		let link = dest_url.to_string();
		let target = link.split(['#', '?']).next().expect("split yields at least one piece");
		let has_scheme = target.split_once(':').is_some_and(|(scheme, _)| !scheme.is_empty() && !scheme.contains('/'));
		if target.is_empty() || has_scheme {
			continue;
		}
		let problem = if target.starts_with('/') {
			Some("is absolute; use a path relative to the file, or a URL")
		} else {
			match normalize(base, target) {
				None => Some("escapes the repository"),
				Some(path) if !(skill_dir.is_empty() || path == skill_dir || path.starts_with(&format!("{skill_dir}/"))) =>
					Some("leaves the skill; anything outside it must be an absolute URL"),
				Some(path) if !(tracked.contains(&path) || tracked.range(format!("{path}/")..).next().is_some_and(|f| f.starts_with(&format!("{path}/")))) =>
					Some("points at nothing tracked"),
				Some(_) => None,
			}
		};
		if let Some(problem) = problem {
			violations.push(Violation::Link {
				file: file.to_owned(),
				link,
				problem,
			});
		}
	}
	violations
}

/// `None` when `..` climbs above the repo root.
fn normalize(base: &str, relative: &str) -> Option<String> {
	let mut parts: Vec<&str> = base.split('/').filter(|p| !p.is_empty()).collect();
	for part in relative.split('/') {
		match part {
			"" | "." => {}
			".." => {
				parts.pop()?;
			}
			p => parts.push(p),
		}
	}
	Some(parts.join("/"))
}
