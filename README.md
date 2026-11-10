# skills
![Lines Of Code](https://img.shields.io/endpoint?url=https://gist.githubusercontent.com/valeratrades/b48e6f02c61942200e7d1e3eeabf9bcb/raw/skills-loc.json)
![Build time](https://img.shields.io/endpoint?url=https://gist.githubusercontent.com/valeratrades/b48e6f02c61942200e7d1e3eeabf9bcb/raw/skills-build.json)
![Test time](https://img.shields.io/endpoint?url=https://gist.githubusercontent.com/valeratrades/b48e6f02c61942200e7d1e3eeabf9bcb/raw/skills-test.json)

The EV team's agent skills, each versioned on its own, and `skargo`: Cargo-style version requirements, a lockfile, and one install for every agent that reads [Agent Skills](https://agentskills.io/specification) (Claude Code, Codex, Kimi Code, opencode, DeepSeek through any of them).
<!-- markdownlint-disable -->
<details>
<summary>
<h2>Installation</h2>
</summary>

With nix: `nix run github:EV-Main/skills -- <command>`, or, in home-manager:
```nix
imports = [ inputs.skargo.homeManagerModules.default ]; # inputs.skargo.url = "github:EV-Main/skills";
programs.skargo = { enable = true; lockFile = <path to your skargo.lock>; };
```

Claude Code only, without skargo: `/plugin marketplace add EV-Main/skills`.


</details>
<!-- markdownlint-restore -->

## Usage
`skargo.toml`:
```toml
skill-creation = "0.1"                                                  # git from the registry, EV-Main/skills
trading-data = "0.1"                                                    # listed in the registry's own skargo.toml
other = { git = "https://github.com/org/repo", version = "1.2" }        # any repo that tags `<name>--v<version>`
v-flakes = { path = "../v_flakes/docs/skill" }                          # a live link to a checkout you're editing
```

```sh
skargo lock              # skargo.toml → skargo.lock, keeping pins it still allows
skargo update [name…]    # re-resolve the named (or all) pins
skargo install           # lock, then link into ~/.claude/skills and ~/.agents/skills
skargo check             # in a skills repo: validate, sync its lock and marketplace, list unreleased tags
skargo publish           # check, then tag HEAD with those and push them
```



<br>

<sup>Links for devs:</sup>
- <sup><a href="./docs/ARCHITECTURE.md">architecture</a></sup>
- <sup><a href="https://github.com/EV-Main/.github/blob/main/code/STYLE.md">style</a></sup>
- <sup><a href="https://github.com/EV-Main/.github/tree/main/code/conventions">conventions</a></sup>
- <sup><a href="https://github.com/EV-Main/.github/blob/main/code/conventions/git">contributing</a></sup>

#### License

<sup>
	Licensed under <a href="LICENSE">Blue Oak 1.0.0</a>
</sup>

<br>

<sub>
	Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be licensed as above, without any additional terms or conditions.
</sub>

