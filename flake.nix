{
  description = "A flake that installs and runs TailwindCSS and Prettier";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable"; # Use the unstable channel
  };

  outputs = {
    self,
    nixpkgs,
  }: let
    system = "x86_64-linux";
    pkgs = nixpkgs.legacyPackages.${system};
  in {
    devShells.${system}.default = pkgs.mkShell {
      nativeBuildInputs = with pkgs; [
        nodePackages.prettier # Prettier
        tailwindcss_4 # Use the unstable version of TailwindCSS
      ];
    };
  };
}
