# skills
![Lines Of Code](https://img.shields.io/endpoint?url=https://gist.githubusercontent.com/valeratrades/b48e6f02c61942200e7d1e3eeabf9bcb/raw/skills-loc.json)

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
`skargo.toml`, any repo that tags `<name>--v<version>`:
```toml
skill-creation = { git = "https://github.com/EV-Main/skills", version = "0.1" }
```

```sh
skargo lock      # skargo.toml → skargo.lock
skargo install   # lock, then link into ~/.claude/skills and ~/.agents/skills
skargo check     # in a skills repo: validate, list unreleased tags
```



<br>

<sup>
	This repository follows <a href="https://github.com/valeratrades/.github/tree/master/best_practices">my best practices</a> and <a href="https://github.com/tigerbeetle/tigerbeetle/blob/main/docs/TIGER_STYLE.md">Tiger Style</a> (except "proper capitalization for acronyms": (VsrState, not VSRState) and formatting). For project's architecture, see <a href="./docs/ARCHITECTURE.md">ARCHITECTURE.md</a>.
</sup>

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

