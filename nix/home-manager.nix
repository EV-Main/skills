# Links the skills `programs.skargo.skills` requires into both agent skill dirs, as `skargo install` does,
# resolved against the marketplace this flake input publishes: `flake.lock` is the only pin.
{ self }:
{ config, lib, pkgs, ... }:
let
  cfg = config.programs.skargo;
  catalogue = lib.listToAttrs (map (p: lib.nameValuePair p.name p) (lib.importJSON "${self}/.claude-plugin/marketplace.json").plugins);

  num = "(0|[1-9][0-9]*)";
  ident = "[0-9A-Za-z-]+";
  pre = "-(${ident}(\\.${ident})*)";
  preIds = s: if s == null then [ ] else lib.splitString "." s;
  leadingZero = ids: lib.any (i: builtins.match "0[0-9]+" i != null) ids;

  # cargo's reading of a bare version: a caret requirement; every other requirement form throws rather than being approximated
  parseReq = name: s:
    let
      m = builtins.match "${num}(\\.${num}(\\.${num}(${pre})?)?)?" s;
      bad = throw "skargo: ${name} = \"${s}\" is not a bare version requirement (`1`, `1.2`, `1.2.3` or `1.2.3-pre`)";
    in
    if m == null || leadingZero (preIds (builtins.elemAt m 6)) then bad else {
      major = builtins.elemAt m 0;
      minor = builtins.elemAt m 2;
      patch = builtins.elemAt m 4;
      pre = preIds (builtins.elemAt m 6);
    };
  parseVersion = name: s:
    let m = builtins.match "${num}\\.${num}\\.${num}(${pre})?(\\+[0-9A-Za-z.-]+)?" s; in
    if m == null then throw "skargo: the marketplace lists ${name} at ${s}, which is not semver" else {
      major = builtins.elemAt m 0;
      minor = builtins.elemAt m 1;
      patch = builtins.elemAt m 2;
      pre = preIds (builtins.elemAt m 4);
    };

  # numbers stay strings, which have no leading zeros: longer is larger
  cmpNum = a: b:
    let la = builtins.stringLength a; lb = builtins.stringLength b; in
    if la != lb then (if la < lb then -1 else 1) else if a == b then 0 else if a < b then -1 else 1;
  isNum = i: builtins.match "[0-9]+" i != null;
  cmpId = a: b:
    if isNum a && isNum b then cmpNum a b
    else if isNum a then -1
    else if isNum b then 1
    else if a == b then 0 else if a < b then -1 else 1;
  cmpIds = a: b:
    if a == [ ] && b == [ ] then 0 else if a == [ ] then -1 else if b == [ ] then 1
    else let c = cmpId (builtins.head a) (builtins.head b); in if c != 0 then c else cmpIds (builtins.tail a) (builtins.tail b);
  preGe = v: r: if r == [ ] then v == [ ] else v == [ ] || cmpIds v r >= 0; # a release outranks its prereleases

  caret = r: v:
    v.major == r.major && (
      if r.minor == null then true
      else if r.patch == null then (if r.major != "0" then cmpNum v.minor r.minor >= 0 else v.minor == r.minor)
      else if r.major != "0" then
        (if v.minor != r.minor then cmpNum v.minor r.minor > 0 else if v.patch != r.patch then cmpNum v.patch r.patch > 0 else preGe v.pre r.pre)
      else if r.minor != "0" then
        v.minor == r.minor && (if v.patch != r.patch then cmpNum v.patch r.patch > 0 else preGe v.pre r.pre)
      else v.minor == r.minor && v.patch == r.patch && preGe v.pre r.pre
    );
  # a prerelease matches only a requirement naming a prerelease of the same version
  matches = r: v: caret r v && (v.pre == [ ] || (r.pre != [ ] && r.major == v.major && r.minor == v.minor && r.patch == v.patch));

  source = name: req:
    if builtins.isAttrs req then config.lib.file.mkOutOfStoreSymlink req.path
    else
      let
        p = catalogue.${name} or (throw "skargo: ${name} is not in the skargo input's marketplace, which lists: ${lib.concatStringsSep ", " (builtins.attrNames catalogue)}. An external skill is listed in the registry's skargo.toml, not here.");
        s = p.source;
      in
      if !matches (parseReq name req) (parseVersion name p.version) then
        throw "skargo: ${name} requires \"${req}\", but the skargo input lists ${p.version}. Raise the requirement, or roll the input back to pin an older release."
      else if builtins.isString s then "${self}/${lib.removePrefix "./" s}"
      else if s.source == "git-subdir" then "${builtins.fetchGit {
        inherit (s) url;
        ref = "refs/tags/${s.ref}";
        rev = s.sha;
        shallow = true;
      }}/${s.path}"
      else throw "skargo: the marketplace sources ${name} from ${s.source}, which this module does not fetch";
in
{
  options.programs.skargo = {
    enable = lib.mkEnableOption "agent skills from the skargo flake input's marketplace";
    skills = lib.mkOption {
      type = lib.types.attrsOf (lib.types.either lib.types.str (lib.types.submodule {
        options.path = lib.mkOption {
          type = lib.types.str;
          description = "absolute path to a checkout's skill dir, linked live";
        };
      }));
      description = "skill name → a bare version requirement, met by the version the input lists, or `{ path }`";
    };
  };

  config = lib.mkIf cfg.enable {
    home.packages = [ self.packages.${pkgs.stdenv.hostPlatform.system}.default ];
    home.file = lib.listToAttrs (lib.concatLists (lib.mapAttrsToList
      (name: req: map (dir: lib.nameValuePair "${dir}/${name}" { source = source name req; }) [ ".claude/skills" ".agents/skills" ])
      cfg.skills));
  };
}
