With nix: `nix run github:EV-Main/skills -- <command>`, or, in home-manager:
```nix
imports = [ inputs.skargo.homeManagerModules.default ]; # inputs.skargo.url = "github:EV-Main/skills";
programs.skargo = { enable = true; lockFile = <path to your skargo.lock>; };
```

Claude Code only, without skargo: `/plugin marketplace add EV-Main/skills`.
