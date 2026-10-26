{pkgs, ...}: let
  surrealdb3 = pkgs.stdenv.mkDerivation {
    pname = "surrealdb";
    version = "3.3.4";

    src = pkgs.fetchurl {
      url = "https://github.com/surrealdb/surrealdb/releases/download/v3.3.4/surreal-v3.3.4.linux-amd64.tgz";
      sha256 = "1296kai9x0rh3q0291lghrn9c27b8cb0pxrwag7qb75d6cc0g4v5";
    };

    nativeBuildInputs = [pkgs.autoPatchelfHook];
    buildInputs = [pkgs.stdenv.cc.cc.lib];

    unpackPhase = ''
      tar xzf $src
    '';

    installPhase = ''
      mkdir -p $out/bin
      install -m755 surreal $out/bin/surreal
    '';
  };
in {
  packages = with pkgs; [
    tailwindcss_4
    clang
    surrealdb3
  ];

  env.SURREALDB_PATH = surrealdb3 + "/bin/surreal";

  git-hooks.hooks.prettier = {
    enable = true;
    types_or = ["markdown"];
  };
}
