//! `SKILL.md` frontmatter per https://agentskills.io/specification, plus our `metadata.version`.

use semver::Version;
use serde_yaml_ng::Value;

#[derive(Debug)]
pub(crate) struct Frontmatter {
	pub(crate) name: String,
	pub(crate) description: String,
	pub(crate) version: Version,
}

/// Every violation found, not just the first.
pub(crate) fn parse(skill_md: &str) -> Result<Frontmatter, Vec<String>> {
	let yaml = match skill_md
		.strip_prefix("---\n")
		.and_then(|rest| rest.split_once("\n---\n").or_else(|| rest.strip_suffix("\n---").map(|y| (y, ""))))
	{
		Some((yaml, _)) => yaml,
		None => return Err(vec!["no `---` delimited frontmatter at the top".to_owned()]),
	};
	let map = match serde_yaml_ng::from_str::<Value>(yaml) {
		Ok(Value::Mapping(map)) => map,
		Ok(_) => return Err(vec!["frontmatter is not a mapping".to_owned()]),
		Err(e) => return Err(vec![format!("frontmatter is not valid YAML: {e}")]),
	};

	let mut violations = Vec::new();
	let mut name = None;
	let mut description = None;
	let mut version = None;
	let mut version_present = false;
	for (key, value) in &map {
		let Some(key) = key.as_str() else {
			violations.push(format!("non-string frontmatter key {key:?}"));
			continue;
		};
		match key {
			"metadata" => {
				let Some(metadata) = value.as_mapping() else {
					violations.push("`metadata` must be a mapping".to_owned());
					continue;
				};
				for (k, v) in metadata {
					match (k.as_str(), v.as_str()) {
						(Some("version"), Some(v)) => {
							version_present = true;
							match Version::parse(v) {
								Ok(v) => version = Some(v),
								Err(e) => violations.push(format!("`metadata.version` {v:?} is not semver: {e}")),
							}
						}
						(Some(_), Some(_)) => {}
						_ => violations.push(format!("`metadata` must map strings to strings, got {k:?}: {v:?}")),
					}
				}
			}
			"name" | "description" | "license" | "compatibility" | "allowed-tools" => {
				let Some(s) = value.as_str() else {
					violations.push(format!("`{key}` must be a string"));
					continue;
				};
				match key {
					"name" => name = Some(s.to_owned()),
					"description" => description = Some(s.to_owned()),
					"compatibility" if s.is_empty() || s.chars().count() > 500 => violations.push("`compatibility` must be 1-500 characters".to_owned()),
					_ => {}
				}
			}
			other => violations.push(format!("`{other}` is not an agentskills spec field")),
		}
	}

	match &name {
		Some(n) => violations.extend(name_violation(n)),
		None => violations.push("`name` is missing".to_owned()),
	}
	match &description {
		Some(d) if d.trim().is_empty() || d.chars().count() > 1024 => violations.push("`description` must be 1-1024 characters".to_owned()),
		Some(_) => {}
		None => violations.push("`description` is missing".to_owned()),
	}
	if !version_present {
		violations.push("`metadata.version` is missing".to_owned());
	}

	match (name, description, version) {
		(Some(name), Some(description), Some(version)) if violations.is_empty() => Ok(Frontmatter { name, description, version }),
		_ => Err(violations),
	}
}

pub(crate) fn name_violation(name: &str) -> Option<String> {
	let valid = (1..=64).contains(&name.len())
		&& name.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
		&& !name.starts_with('-')
		&& !name.ends_with('-')
		&& !name.contains("--");
	(!valid).then(|| format!("name {name:?} must be 1-64 of [a-z0-9-], without leading, trailing or doubled hyphens"))
}
