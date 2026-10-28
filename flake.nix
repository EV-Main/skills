{
  nixConfig = {
    extra-substituters = [ "https://valeratrades.cachix.org" ];
    extra-trusted-public-keys = [ "valeratrades.cachix.org-1:gXVwhzO5YB+BaiEJYT48qZgzdaErGQew6xtZcz4Fo1Q=" ];
  };

  inputs = {
    v_flakes.url = "github:valeratrades/v_flakes?ref=v1.6";
  };

  outputs = { self, v_flakes }:
    let
      inherit (v_flakes) flake-utils pre-commit-hooks;
      pname = "skills";
      manifest = (v_flakes.nixpkgs.lib.importTOML ./Cargo.toml).package;
    in
    flake-utils.lib.eachDefaultSystem
      (
        system:
        let
          pkgs = import v_flakes.default_nixpkgs { inherit system; };
          rust = v_flakes.rs.default_nightly system;
          stdenv = pkgs.stdenvAdapters.useMoldLinker pkgs.stdenv;
          pre-commit-check = pre-commit-hooks.lib.${system}.run (v_flakes.files.preCommit {
            inherit pkgs;
            webAssets = { };
          });

          rs = v_flakes.rs {
            inherit pkgs rust;
            build.enable = false;
          };
          github = v_flakes.github {
            inherit pkgs pname rs;
            enable = true;
            lastSupportedVersion = "nightly-${v_flakes.rs.nightly_version}";
            jobs.default = true;
            jobs.errors.install = { packages = [ "mold" "git" ]; }; # brings nix itself: tests check the home-manager module against the lock
            jobs.warnings.install = { packages = [ "mold" ]; debug = true; };
          };
          readme = v_flakes.readme-fw {
            inherit pkgs pname;
            defaults = true;
            lastSupportedVersion = "";
            rootDir = ./.;
            badges = [ "loc" ];
          };
          combined = v_flakes.utils.combine { inherit rust; modules = [ rs github readme ]; };
        in
        {
          apps.help = v_flakes.utils.help { inherit pkgs self system; };

          packages.default =
            let build_rust = v_flakes.rs.build_nightly system; in
            (pkgs.makeRustPlatform { rustc = build_rust; cargo = build_rust; inherit stdenv; }).buildRustPackage {
              pname = manifest.name;
              version = manifest.version;
              src = pkgs.lib.cleanSource ./.;
              cargoLock.lockFile = ./Cargo.lock;
              RUSTC_WRAPPER = ""; # .cargo/config.toml sets sccache, absent in the sandbox
              doCheck = false; # tests drive git and nix against fixture repos; CI runs them
              nativeBuildInputs = [ pkgs.makeWrapper ];
              postInstall = "wrapProgram $out/bin/${manifest.name} --prefix PATH : ${pkgs.lib.makeBinPath [ pkgs.git ]}";
            };

          devShells.default = pkgs.mkShell {
            inherit stdenv;
            shellHook =
              pre-commit-check.shellHook
              + combined.shellHook
              + ''
                cp -f ${(v_flakes.files.treefmt) { inherit pkgs; }} ./.treefmt.toml
              '';

            packages = [ pkgs.mold rust ] ++ pre-commit-check.enabledPackages ++ combined.enabledPackages;
          };
        }
      )
    // {
      homeManagerModules.default = import ./nix/home-manager.nix { inherit self; };
    };
}
