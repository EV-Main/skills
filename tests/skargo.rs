//! Drives the `skargo` binary against throwaway git repos: what a consumer or publisher sees, not how it's done.

use std::{
	collections::BTreeMap,
	fs,
	os::unix::fs::symlink,
	path::{Path, PathBuf},
	process::{Command, Output},
};

use tempfile::TempDir;

/// What goes wrong, how to break a repo that way, what the error must say.
type Case = (&'static str, fn(&Repo), &'static str);

/// Keeps fixtures and the binary under test away from the user's git config (signing, url rewrites, hooks).
fn isolated(mut c: Command) -> Command {
	c.env("GIT_CONFIG_GLOBAL", "/dev/null")
		.env("GIT_CONFIG_NOSYSTEM", "1")
		.env("GIT_AUTHOR_NAME", "t")
		.env("GIT_AUTHOR_EMAIL", "t@t")
		.env("GIT_COMMITTER_NAME", "t")
		.env("GIT_COMMITTER_EMAIL", "t@t");
	c
}

fn git(dir: &Path, args: &[&str]) -> String {
	let mut c = isolated(Command::new("git"));
	let out = c.arg("-C").arg(dir).args(args).output().expect("git on PATH");
	assert!(out.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&out.stderr));
	String::from_utf8(out.stdout).unwrap().trim().to_owned()
}

/// A skills source repo.
struct Repo(TempDir);
impl Repo {
	fn new() -> Self {
		let repo = Self(tempfile::tempdir().unwrap());
		git(repo.path(), &["init", "-q", "-b", "main"]);
		repo
	}

	fn path(&self) -> &Path {
		self.0.path()
	}

	fn url(&self) -> String {
		format!("file://{}", self.path().display())
	}

	fn write(&self, file: &str, content: &str) {
		let path = self.path().join(file);
		fs::create_dir_all(path.parent().unwrap()).unwrap();
		fs::write(path, content).unwrap();
	}

	fn commit(&self) -> String {
		git(self.path(), &["add", "-A"]);
		git(self.path(), &["commit", "-q", "--allow-empty", "-m", "c"]);
		git(self.path(), &["rev-parse", "HEAD"])
	}

	/// Commits `<dir>/SKILL.md` declaring `name`@`version` and tags it `<name>--v<version>`; returns the commit.
	fn release(&self, dir: &str, name: &str, version: &str) -> String {
		self.write(&format!("{dir}/SKILL.md"), &skill_md(name, version));
		let commit = self.commit();
		git(self.path(), &["tag", &format!("{name}--v{version}")]);
		commit
	}
}

fn skill_md(name: &str, version: &str) -> String {
	format!("---\nname: {name}\ndescription: test skill {name}\nmetadata:\n  version: \"{version}\"\n---\n\nbody of {name} {version}\n")
}

/// A consumer machine: its own HOME, data dir and working dir.
struct Machine(TempDir);

impl Machine {
	fn new() -> Self {
		let m = Self(tempfile::tempdir().unwrap());
		fs::create_dir_all(m.work()).unwrap();
		m
	}

	fn home(&self) -> PathBuf {
		self.0.path().join("home")
	}

	fn work(&self) -> PathBuf {
		self.0.path().join("work")
	}

	fn store(&self) -> PathBuf {
		self.0.path().join("data/skargo")
	}

	fn manifest(&self, toml: &str) {
		fs::write(self.work().join("skargo.toml"), toml).unwrap();
	}

	fn lock_text(&self) -> String {
		fs::read_to_string(self.work().join("skargo.lock")).unwrap()
	}

	fn locked(&self, name: &str) -> (String, String, String) {
		let lock: toml::Table = self.lock_text().parse().unwrap();
		let entry = lock["skill"].as_array().unwrap().iter().find(|e| e["name"].as_str() == Some(name)).unwrap();
		(
			entry["version"].as_str().unwrap().to_owned(),
			entry["commit"].as_str().unwrap().to_owned(),
			entry["path"].as_str().unwrap().to_owned(),
		)
	}

	fn run(&self, args: &[&str]) -> Output {
		skargo(&self.work(), args, |c| {
			c.env("HOME", self.home()).env("XDG_DATA_HOME", self.0.path().join("data"));
		})
	}
}

fn skargo(cwd: &Path, args: &[&str], configure: impl FnOnce(&mut Command)) -> Output {
	let mut c = isolated(Command::new(env!("CARGO_BIN_EXE_skargo")));
	c.current_dir(cwd).args(args).env("NO_COLOR", "1");
	configure(&mut c);
	c.output().unwrap()
}

#[track_caller]
fn ok(out: Output) -> String {
	assert!(out.status.success(), "expected success, stderr:\n{}", String::from_utf8_lossy(&out.stderr));
	String::from_utf8(out.stdout).unwrap()
}

#[track_caller]
fn fails_with(out: Output, fragment: &str) {
	let stderr = String::from_utf8_lossy(&out.stderr);
	assert!(!out.status.success(), "expected failure mentioning {fragment:?}, it succeeded");
	let unwrapped = stderr.replace('│', " ").split_whitespace().collect::<Vec<_>>().join(" "); // miette wraps to the terminal width
	assert!(unwrapped.contains(fragment), "expected {fragment:?} in stderr:\n{stderr}");
}

#[test]
fn requirements_resolve_with_cargo_semantics() {
	let repo = Repo::new();
	let mut commits = std::collections::HashMap::new();
	for v in ["1.0.0", "1.1.0", "1.1.5", "1.2.0", "2.0.0", "2.1.0-beta.1"] {
		commits.insert(v, repo.release("skills/foo", "foo", v));
	}
	repo.write("skills/foo/SKILL.md", &skill_md("foo", "2.0.1"));
	commits.insert("2.0.1", repo.commit());
	git(repo.path(), &["tag", "-a", "-m", "annotated", "foo--v2.0.1"]);

	let m = Machine::new();
	let cases: &[(&str, Option<&str>)] = &[
		("1", Some("1.2.0")),
		("1.1", Some("1.2.0")),
		("^1.1", Some("1.2.0")),
		("~1.1.0", Some("1.1.5")),
		("~1.1", Some("1.1.5")),
		("=1.1.0", Some("1.1.0")),
		(">=1.1, <1.2", Some("1.1.5")),
		("2", Some("2.0.1")),
		("^2.1.0-0", Some("2.1.0-beta.1")),
		("*", Some("2.0.1")),
		("3", None),
	];
	for (req, expected) in cases {
		let lock = m.work().join("skargo.lock");
		if lock.exists() {
			fs::remove_file(lock).unwrap();
		}
		m.manifest(&format!("foo = {{ git = \"{}\", version = \"{req}\" }}\n", repo.url()));
		let out = m.run(&["lock"]);
		match expected {
			Some(v) => {
				ok(out);
				assert_eq!(m.locked("foo"), (v.to_string(), commits[v].clone(), "skills/foo".to_owned()), "requirement {req:?}");
			}
			None => fails_with(out, "no release of foo"),
		}
	}
}

#[test]
fn lock_keeps_pins_the_manifest_still_allows() {
	let repo = Repo::new();
	repo.release("skills/foo", "foo", "1.0.0");
	let v1_1 = repo.release("skills/foo", "foo", "1.1.0");
	let m = Machine::new();
	let manifest = |req: &str| format!("foo = {{ git = \"{}\", version = \"{req}\" }}\n", repo.url());
	m.manifest(&manifest("1"));
	ok(m.run(&["lock"]));
	let first = m.lock_text();
	ok(m.run(&["lock"]));
	assert_eq!(m.lock_text(), first, "relocking unchanged inputs must not touch the lock");

	let v1_2 = repo.release("skills/foo", "foo", "1.2.0");
	ok(m.run(&["lock"]));
	assert_eq!(m.locked("foo").1, v1_1, "a newer matching release must not move an existing pin");

	m.manifest(&manifest("=1.0.0"));
	ok(m.run(&["lock"]));
	assert_eq!(m.locked("foo").0, "1.0.0", "a requirement the pin no longer meets re-resolves");

	m.manifest(&manifest("1"));
	fs::remove_file(m.work().join("skargo.lock")).unwrap();
	ok(m.run(&["lock"]));
	assert_eq!(m.locked("foo").1, v1_2, "without a lock, the newest match wins");
}

#[test]
fn moved_tag_is_refused() {
	let repo = Repo::new();
	repo.release("skills/foo", "foo", "1.0.0");
	let m = Machine::new();
	m.manifest(&format!("foo = {{ git = \"{}\", version = \"1\" }}\n", repo.url()));
	ok(m.run(&["lock"]));
	repo.write("skills/foo/extra.md", "x");
	repo.commit();
	git(repo.path(), &["tag", "-f", "foo--v1.0.0"]);
	fails_with(m.run(&["lock"]), "moved");
}

#[test]
fn lock_derives_the_skill_path_from_its_name() {
	let repo = Repo::new();
	repo.write("skills/bar/SKILL.md", &skill_md("bar", "0.1.0"));
	repo.release("docs/skill", "foo", "0.3.0");
	let m = Machine::new();
	m.manifest(&format!("foo = {{ git = \"{}\", version = \"0.3\" }}\n", repo.url()));
	ok(m.run(&["lock"]));
	assert_eq!(m.locked("foo").2, "docs/skill");

	let cases: &[Case] = &[
		(
			"no SKILL.md carries the name",
			|r| r.write("docs/skill/SKILL.md", &skill_md("other", "1.0.0")),
			"no SKILL.md named \"foo\"",
		),
		(
			"two SKILL.md carry the name",
			|r| {
				r.write("a/SKILL.md", &skill_md("foo", "1.0.0"));
				r.write("b/SKILL.md", &skill_md("foo", "1.0.0"));
			},
			"several SKILL.md",
		),
		(
			"metadata.version disagrees with the tag",
			|r| r.write("docs/skill/SKILL.md", &skill_md("foo", "1.0.1")),
			"the tag says 1.0.0",
		),
	];
	for (what, setup, fragment) in cases {
		let repo = Repo::new();
		setup(&repo);
		repo.commit();
		git(repo.path(), &["tag", "foo--v1.0.0"]);
		let m = Machine::new();
		m.manifest(&format!("foo = {{ git = \"{}\", version = \"1\" }}\n", repo.url()));
		let out = m.run(&["lock"]);
		assert!(!out.status.success(), "{what}: lock should fail");
		fails_with(out, fragment);
	}
}

#[test]
fn install_links_locked_skills_for_every_agent() {
	let repo = Repo::new();
	repo.release("skills/foo", "foo", "1.0.0");
	repo.release("docs/skill", "bar", "0.2.0");
	repo.release("skills/foo", "foo", "1.1.0");
	let m = Machine::new();
	let both = format!("foo = {{ git = \"{0}\", version = \"=1.0.0\" }}\nbar = {{ git = \"{0}\", version = \"0.2\" }}\n", repo.url());
	m.manifest(&both);
	ok(m.run(&["install"]));
	for agent_dir in [".claude/skills", ".agents/skills"] {
		let dir = m.home().join(agent_dir);
		assert_eq!(fs::read_to_string(dir.join("foo/SKILL.md")).unwrap(), skill_md("foo", "1.0.0"));
		assert_eq!(fs::read_to_string(dir.join("bar/SKILL.md")).unwrap(), skill_md("bar", "0.2.0"));
		assert!(dir.join("foo").symlink_metadata().unwrap().file_type().is_symlink(), "a whole-dir link, which Codex follows");
	}

	m.manifest(&format!("foo = {{ git = \"{}\", version = \"1.1\" }}\n", repo.url()));
	ok(m.run(&["install"]));
	for agent_dir in [".claude/skills", ".agents/skills"] {
		let dir = m.home().join(agent_dir);
		assert!(dir.join("bar").symlink_metadata().is_err(), "dropped from the manifest, so unlinked");
		assert_eq!(fs::read_to_string(dir.join("foo/SKILL.md")).unwrap(), skill_md("foo", "1.1.0"));
	}
	assert_eq!(fs::read_dir(m.store()).unwrap().count(), 1, "unreferenced store entries are collected");
}

#[test]
fn install_leaves_foreign_skills_alone() {
	let repo = Repo::new();
	repo.release("skills/foo", "foo", "1.0.0");
	let m = Machine::new();
	let mine = m.home().join(".claude/skills/foo");
	fs::create_dir_all(&mine).unwrap();
	fs::write(mine.join("SKILL.md"), "hand made").unwrap();
	m.manifest(&format!("foo = {{ git = \"{}\", version = \"1\" }}\n", repo.url()));
	fails_with(m.run(&["install"]), "not installed by skargo");
	assert_eq!(fs::read_to_string(mine.join("SKILL.md")).unwrap(), "hand made");
	assert!(!m.home().join(".agents/skills/foo").exists(), "nothing is written when any target is foreign");
}

/// A publishable repo with one linked reference file, its marketplace in sync.
fn clean_source() -> Repo {
	let repo = Repo::new();
	repo.write(
		"skills/a/SKILL.md",
		&format!("{}\nSee [ref](references/r.md) and [site](https://x.dev).\n", skill_md("a", "0.1.0")),
	);
	repo.write("skills/a/references/r.md", "back to [top](../SKILL.md#principles)\n");
	repo.write(
		".claude-plugin/marketplace.json",
		r#"{
  "name": "m",
  "owner": {
    "name": "o"
  },
  "plugins": [
    {
      "name": "a",
      "source": "./skills/a",
      "description": "test skill a",
      "version": "0.1.0"
    }
  ]
}
"#,
	);
	repo.commit();
	repo
}

fn check(repo: &Repo) -> Output {
	skargo(repo.path(), &["check"], |_| {})
}

#[test]
fn check_passes_a_clean_repo_and_prints_unreleased_tags() {
	let repo = clean_source();
	assert_eq!(ok(check(&repo)), "a--v0.1.0\n");
	git(repo.path(), &["tag", "a--v0.1.0"]);
	assert_eq!(ok(check(&repo)), "", "released and unchanged");
}

#[test]
fn check_regenerates_the_marketplace() {
	let repo = clean_source();
	repo.write("skills/b/SKILL.md", &skill_md("b", "1.0.0"));
	repo.commit();
	fails_with(check(&repo), "marketplace.json was stale");
	let marketplace: serde_json::Value = serde_json::from_str(&fs::read_to_string(repo.path().join(".claude-plugin/marketplace.json")).unwrap()).unwrap();
	assert_eq!(marketplace["name"], "m", "hand-written fields survive");
	assert_eq!(marketplace["plugins"][1]["source"], "./skills/b");
	assert_eq!(marketplace["plugins"][1]["version"], "1.0.0");
	repo.commit();
	ok(check(&repo));
}

#[test]
fn check_reports_each_violation() {
	let cases: &[Case] = &[
		(
			"non-spec field",
			|r| r.write("skills/a/SKILL.md", &skill_md("a", "0.1.0").replacen("name:", "version: 1\nname:", 1)),
			"`version` is not an agentskills spec field",
		),
		(
			"missing version",
			|r| r.write("skills/a/SKILL.md", "---\nname: a\ndescription: d\n---\n"),
			"`metadata.version` is missing",
		),
		("non-semver version", |r| r.write("skills/a/SKILL.md", &skill_md("a", "1.0")), "is not semver"),
		(
			"invalid name",
			|r| r.write("skills/a/SKILL.md", &skill_md("a--b", "0.1.0")),
			"without leading, trailing or doubled hyphens",
		),
		("nested skill", |r| r.write("skills/a/sub/SKILL.md", &skill_md("sub", "0.1.0")), "sits inside skill skills/a/"),
		(
			"duplicate name",
			|r| r.write("skills/b/SKILL.md", &skill_md("a", "0.1.0")),
			"is the frontmatter name of several skills",
		),
		("link leaving the skill", |r| r.write("skills/a/references/r.md", "[x](../../../README.md)\n"), "leaves the skill"),
		("dangling link", |r| r.write("skills/a/references/r.md", "[x](missing.md)\n"), "points at nothing tracked"),
		("absolute link", |r| r.write("skills/a/references/r.md", "[x](/etc/passwd)\n"), "is absolute"),
		("symlink", |r| symlink("../SKILL.md", r.path().join("skills/a/references/link")).unwrap(), "is a symlink"),
		(
			"changed without bump",
			|r| {
				git(r.path(), &["tag", "a--v0.1.0"]);
				r.write("skills/a/references/r.md", "changed\n");
			},
			"without a version bump",
		),
		(
			"version not above latest",
			|r| {
				git(r.path(), &["tag", "a--v0.2.0"]);
			},
			"must exceed latest released 0.2.0",
		),
		(
			"malformed tag",
			|r| {
				git(r.path(), &["tag", "a--vlatest"]);
			},
			"is not <name>--v<semver>",
		),
	];
	for (what, mutate, fragment) in cases {
		let repo = clean_source();
		mutate(&repo);
		repo.commit();
		let out = check(&repo);
		assert!(!out.status.success(), "{what}: check should fail");
		fails_with(out, fragment);
	}
}

/// The lock is the contract between the CLI and the home-manager module.
#[test]
fn home_manager_module_links_what_the_lock_pins() {
	let repo = Repo::new();
	repo.release("skills/foo", "foo", "1.0.0");
	repo.release("docs/skill", "bar", "0.2.0");
	let m = Machine::new();
	m.manifest(&format!("foo = {{ git = \"{0}\", version = \"1\" }}\nbar = {{ git = \"{0}\", version = \"0.2\" }}\n", repo.url()));
	ok(m.run(&["lock"]));

	let root = env!("CARGO_MANIFEST_DIR");
	let expr = format!(
		r#"
		let
			lib = (builtins.getFlake "git+file://{root}?shallow=1").inputs.v_flakes.nixpkgs.lib;
			stub = {{ lib, ... }}: {{
				options.home.file = lib.mkOption {{ type = lib.types.attrsOf lib.types.anything; }};
				options.home.packages = lib.mkOption {{ type = lib.types.listOf lib.types.anything; }};
			}};
			eval = lib.evalModules {{
				modules = [
					(import {root}/nix/home-manager.nix {{ self.packages.test.default = "skargo"; }})
					stub
					{{ programs.skargo = {{ enable = true; lockFile = {lock}; }}; }}
				];
				specialArgs.pkgs.stdenv.hostPlatform.system = "test";
			}};
		in builtins.mapAttrs (_: f: f.source) eval.config.home.file
		"#,
		lock = m.work().join("skargo.lock").display(),
	);
	let out = Command::new("nix").args(["eval", "--impure", "--json", "--expr", &expr]).output().expect("nix on PATH");
	assert!(out.status.success(), "nix eval: {}", String::from_utf8_lossy(&out.stderr));
	let files: BTreeMap<String, String> = serde_json::from_slice(&out.stdout).unwrap();
	assert_eq!(
		files.keys().map(String::as_str).collect::<Vec<_>>(),
		[".agents/skills/bar", ".agents/skills/foo", ".claude/skills/bar", ".claude/skills/foo"]
	);
	assert_eq!(fs::read_to_string(Path::new(&files[".claude/skills/foo"]).join("SKILL.md")).unwrap(), skill_md("foo", "1.0.0"));
	assert_eq!(fs::read_to_string(Path::new(&files[".agents/skills/bar"]).join("SKILL.md")).unwrap(), skill_md("bar", "0.2.0"));
}
