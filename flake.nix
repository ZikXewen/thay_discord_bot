{
  description = "Flake for development and running the scripts";
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
  };
  outputs = {
    self,
    nixpkgs,
  }: let
    pkgs = nixpkgs.legacyPackages."x86_64-linux";
  in {
    devShells."x86_64-linux".default = pkgs.mkShell {
      packages = [
        pkgs.cargo
        pkgs.rustc
        (pkgs.python3.withPackages (python-pkgs: with python-pkgs; [requests]))
      ];
    };
  };
}
