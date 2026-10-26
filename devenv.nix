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
    (cargo-llvm-cov.overrideAttrs (_: {doCheck = false;}))
  ];

  env.SURREALDB_PATH = surrealdb3 + "/bin/surreal";
  env.LLVM_COV = "${pkgs.llvmPackages_23.llvm}/bin/llvm-cov";
  env.LLVM_PROFDATA = "${pkgs.llvmPackages_23.llvm}/bin/llvm-profdata";

  git-hooks.hooks.prettier = {
    enable = true;
    types_or = ["markdown"];
  };
}
