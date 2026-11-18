With nix: `nix run github:EV-Main/skills -- <command>`, or, in home-manager:
```nix
imports = [ inputs.skargo.homeManagerModules.default ]; # inputs.skargo.url = "github:EV-Main/skills?ref=cached";
programs.skargo = { enable = true; skills = { skill-creation = "0.3"; v-flakes.path = "/abs/v_flakes/docs/skill"; }; }; # `nix flake update skargo` bumps them
```

Claude Code only, without skargo: `/plugin marketplace add EV-Main/skills`.
