{pkgs, ...}: let
  surrealdb3 = pkgs.stdenv.mkDerivation {
    pname = "surrealdb";
    version = "3.1.4";

    src = pkgs.fetchurl {
      url = "https://github.com/surrealdb/surrealdb/releases/download/v3.1.4/surreal-v3.1.4.linux-amd64.tgz";
      sha256 = "1f5xbkx5x2pr6xvf3yzwkj9f6y5ln977bjk2x2w3bp94h16rj9zz";
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
}
