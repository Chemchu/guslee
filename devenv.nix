{pkgs, ...}: {
  packages = with pkgs; [
    tailwindcss_4
    clang
    surrealdb
  ];

  /*
  env.SURREALDB_PATH = "/home/gus/.surrealdb/surreal";
  */
  env.SURREALDB_PATH = pkgs.surrealdb + "/bin/surreal";
}
