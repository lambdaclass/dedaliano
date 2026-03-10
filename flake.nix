{
  description = "Dedaliano – structural analysis web app";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          targets = [ "wasm32-unknown-unknown" ];
        };
      in {
        devShells.default = pkgs.mkShell {
          buildInputs = [
            rustToolchain
            pkgs.wasm-pack
            pkgs.nodejs_20
            pkgs.wasm-bindgen-cli
          ];

          shellHook = ''
            echo "Dedaliano dev shell ready"
            echo "  Rust: $(rustc --version)"
            echo "  Node: $(node --version)"
            echo "  wasm-pack: $(wasm-pack --version)"
          '';
        };

        # Build the WASM engine
        packages.wasm-engine = pkgs.stdenv.mkDerivation {
          pname = "dedaliano-engine-wasm";
          version = "0.1.0";
          src = ./engine;

          nativeBuildInputs = [ rustToolchain pkgs.wasm-pack pkgs.wasm-bindgen-cli ];

          buildPhase = ''
            HOME=$(mktemp -d)
            wasm-pack build --target web --out-dir $out
          '';

          dontInstall = true;
        };
      }
    );
}
