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
