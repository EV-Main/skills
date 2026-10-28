`skargo.toml`, any repo that tags `<name>--v<version>`:
```toml
skill-creation = { git = "https://github.com/EV-Main/skills", version = "0.1" }
```

```sh
skargo lock      # skargo.toml → skargo.lock
skargo install   # lock, then link into ~/.claude/skills and ~/.agents/skills
skargo check     # in a skills repo: validate, list unreleased tags
```
