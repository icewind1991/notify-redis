{
  rustPlatform,
  lib,
}: let
  src = lib.sources.sourceByRegex (lib.cleanSource ./.) ["Cargo.*" "(src|tests)(/.*)?"];
in
  rustPlatform.buildRustPackage rec {
    version = "0.2.1";
    pname = "notify-redis";

    inherit src;

    cargoLock = {
      lockFile = ./Cargo.lock;
    };

    doCheck = false;

    meta = with lib; {
      description = "Push filesystem notifications into a redis list";
      homepage = "https://github.com/icewind1991/notify-redis";
      license = licenses.mit;
      platforms = platforms.linux;
    };
  }
