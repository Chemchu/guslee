{
  description = "A flake that installs and runs tailwindcss if not already installed";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
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
        curl
        (writeShellScriptBin "check-and-install-tailwindcss" ''
          if ! command -v ./tailwindcss &> /dev/null; then
            echo "TailwindCSS not found. Installing..."
            curl -sLO https://github.com/tailwindlabs/tailwindcss/releases/latest/download/tailwindcss-linux-x64
            chmod +x tailwindcss-linux-x64
            mv tailwindcss-linux-x64 tailwindcss
            echo "TailwindCSS installed!"
          else
            echo "TailwindCSS is already installed."
          fi
        '')
      ];

      shellHook = ''
        check-and-install-tailwindcss
      '';
    };
  };
}
