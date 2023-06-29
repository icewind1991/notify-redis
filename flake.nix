{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "nixpkgs/nixos-23.05";
    naersk.url = "github:nix-community/naersk";
    naersk.inputs.nixpkgs.follows = "nixpkgs";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
    rust-overlay.inputs.flake-utils.follows = "flake-utils";
    cross-naersk.url = "github:icewind1991/cross-naersk";
    cross-naersk.inputs.nixpkgs.follows = "nixpkgs";
    cross-naersk.inputs.naersk.follows = "naersk";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    naersk,
    rust-overlay,
    cross-naersk,
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        overlays = [(import rust-overlay)];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        lib = pkgs.lib;

        cross-naersk' = pkgs.callPackage cross-naersk {inherit naersk;};

        hostTarget = pkgs.hostPlatform.config;
        targets = [
          "x86_64-unknown-linux-musl"
          "i686-unknown-linux-musl"
          "armv7-unknown-linux-musleabihf"
          "aarch64-unknown-linux-musl"
        ];
        src = lib.sources.sourceByRegex (lib.cleanSource ./.) ["Cargo.*" "(src|tests)(/.*)?"];

        nearskOpt = {
          pname = "notify-redis";
          root = src;
        };
        buildTarget = target: (cross-naersk'.buildPackage target) nearskOpt;
        hostNaersk = cross-naersk'.hostNaersk;
      in rec {
        # `nix build`
        packages = nixpkgs.lib.attrsets.genAttrs targets buildTarget // rec {
          notify-redis = pkgs.callPackage (import ./package.nix) {};
          default = notify-redis;
          check = hostNaersk.buildPackage (nearskOpt // {
            mode = "check";
          });
          clippy = hostNaersk.buildPackage (nearskOpt // {
            mode = "clippy";
          });
          test = hostNaersk.buildPackage (nearskOpt // {
            mode = "test";
            nativeBuildInputs = [pkgs.redis];
            overrideMain = x: x // {
              preBuild = ''
                redis-server &
                export redisPID=$!
              '';
              postBuild = ''
                kill $redisPID
              '';
            };
          });
          dockerImage = pkgs.dockerTools.buildImage {
            name = "icewind1991/notify-redis";
            tag = "latest";
            copyToRoot = [notify-redis];
            config = {
              Cmd = ["${notify-redis}/bin/notify-redis"];
            };
          };
        };

        inherit targets;

        # `nix develop`
        devShells.default = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [rustc cargo bacon cargo-edit cargo-outdated clippy];
        };
      }
    );
}
