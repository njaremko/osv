{
  description = "Central repository of all giza builds";
  nixConfig = {
    max-jobs = 32;
    http-connections = 128;
    max-substitution-jobs = 128;
    substituters = [
      "https://cache.nixos.org?priority=1"
      "https://nix-community.cachix.org?priority=2"
    ];
    trusted-public-keys = [
      "cache.nixos.org-1:6NCHdD59X431o0gWypbMrAURkbJ16ZPMQFGspcDShjY="
      "nix-community.cachix.org-1:mB9FSh9qf2dCimDSUo8Zy7bkq5CX+/rkCWyvRCYg3Fs="
    ];
    # This setting, when true, tries to use symlinks to optimise storage use between nix derivations.
    # However, on MacOS, it sometimes runs into issues, and causes stuff to build from scratch...
    # Which is strictly worse than using some extra storage sometimes. So we'll force it to false.
    auto-optimise-store = false;
  };
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    flake-parts.url = "github:hercules-ci/flake-parts";
    napalm = {
      url = "github:nix-community/napalm";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
    nixpkgs.url = "github:NixOS/nixpkgs/master";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
    crane = {
      url = "github:ipetkov/crane";
    };
  };
  outputs = inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [ "x86_64-linux" "aarch64-linux" "aarch64-darwin" "x86_64-darwin" ];
      imports = [

      ];

      perSystem = { config, self', inputs', pkgs, system, ... }:
        let
          linuxSystem = builtins.replaceStrings [ "darwin" ] [ "linux" ] system;
        in
        {
          _module.args.linuxSystem = linuxSystem;
          _module.args.pkgs = import inputs.nixpkgs {
            inherit system;
            overlays = import ./overlay.nix inputs;
            config = {
              allowUnfree = true;
              allowUnsupportedSystem = true;
              permittedInsecurePackages = [
                "openssl-1.1.1w"
              ];
            };
          };
          _module.args.pkgsLinux = import inputs.nixpkgs {
            system = linuxSystem;
            overlays = import ./overlay.nix inputs;
            config = {
              allowUnfree = true;
              allowUnsupportedSystem = true;
              permittedInsecurePackages = [
                "openssl-1.1.1w"
              ];
            };
          };
          legacyPackages.nixpkgs = pkgs;
          devShells.default = pkgs.mkShell {
            packages = with pkgs;[
              ruby_3_3
              bundler
              rust-analyzer-unwrapped
              rust-dev-toolchain
            ];
          };
        };
    };
}
