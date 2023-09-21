{
  description = "A very basic flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
      in {
        formatter = pkgs.nixfmt;

        devShells = {
          default = pkgs.mkShell {
            buildInputs = with pkgs;
              [
              rust-bin.nightly.latest.default
              gnumake
              ]
              ++ pkgs.lib.optional pkgs.stdenv.isDarwin pkgs.libiconv;

            shellHook = ''
              alias ls=exa
              alias find=fd
            '';
          };
          ktest_clang = null;
          ktest_gcc = null;
        };
      });
}
