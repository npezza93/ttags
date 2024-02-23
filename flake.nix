{
  description = "ttags generates tags using tree-sitter";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    pre-commit-hooks = {
      url = "github:cachix/pre-commit-hooks.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs @ {
    self,
    nixpkgs,
    flake-parts,
    pre-commit-hooks,
    ...
  }: let
    get-version = pkgs: ((pkgs.lib.importTOML "${self}/Cargo.toml").package).version;
  in
    flake-parts.lib.mkFlake {inherit inputs;} {
      systems = [
        "x86_64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      perSystem = {
        config,
        self',
        inputs',
        pkgs,
        system,
        ...
      }: let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [
            self.overlays.default
          ];
        };
        pre-commit-check = pre-commit-hooks.lib.${system}.run {
          src = self;
          hooks = {
            # FIXME: disabled, because it fails
            # rustfmt.enable = true;
            alejandra.enable = true;
            editorconfig-checker.enable = true;
          };
        };
      in {
        packages = rec {
          default = ttags;
          inherit (pkgs) ttags;
        };
        devShells.default = pkgs.mkShell {
          name = "ttags devShell";
          buildInputs = with pkgs;
          with pkgs.rustPlatform;
            [
              cargo
              rustc
              rust-analyzer
            ]
            ++ (with pre-commit-hooks.packages.${system}; [
              rustfmt
              alejandra
              editorconfig-checker
            ]);
          shellHook = ''
            # pre-commit hooks can be disabled by adding
            # a .no-pre-commit file
            if [ ! -f .no-pre-commit ]; then
              ${self.checks.${system}.pre-commit-check.shellHook}
            fi
          '';
        };
        checks = {
          inherit pre-commit-check;
          version = pkgs.testers.testVersion {
            package = pkgs.ttags;
            command = "ttags --version";
            version = get-version pkgs;
          };
        };
      };

      flake = {
        overlays.default = final: prev: {
          ttags = prev.ttags.overrideAttrs (oa: {
            src = self;
            version = get-version prev;
            cargoDeps = prev.rustPlatform.importCargoLock {
              lockFile = self + "/Cargo.lock";
            };
          });
        };
      };
    };
}
