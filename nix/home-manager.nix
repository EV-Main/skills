# Links every skill pinned by a `skargo.lock` into both agent skill dirs, as `skargo install` does.
{ self }:
{ config, lib, pkgs, ... }:
let
  cfg = config.programs.skargo;
  skills = (builtins.fromTOML (builtins.readFile cfg.lockFile)).skill;
  source = s: "${builtins.fetchGit {
    url = s.git;
    ref = "refs/tags/${s.name}--v${s.version}";
    rev = s.commit;
    shallow = true;
  }}/${s.path}";
in
{
  options.programs.skargo = {
    enable = lib.mkEnableOption "agent skills pinned by a skargo.lock";
    lockFile = lib.mkOption {
      type = lib.types.path;
      description = "`skargo.lock` written by `skargo lock`";
    };
  };

  config = lib.mkIf cfg.enable {
    home.packages = [ self.packages.${pkgs.stdenv.hostPlatform.system}.default ];
    home.file = lib.listToAttrs (lib.concatMap
      (s: map (dir: lib.nameValuePair "${dir}/${s.name}" { source = source s; }) [ ".claude/skills" ".agents/skills" ])
      skills);
  };
}
