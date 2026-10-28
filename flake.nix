{
  description = "ESP32-C3 Retro Arcade Game Stack Environment (Nix Flake)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          targets = [ "riscv32imc-unknown-none-elf" ];
          extensions = [ "rust-src" "rust-analyzer" ];
        };
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustToolchain
            espflash
            pkg-config
            libusb1
            python3
          ];

          shellHook = ''
            echo "ESP32-C3 Arcade Environment Ready (Target: riscv32imc-unknown-none-elf)"
            echo "Commands:"
            echo "  cargo build --release --bin arcade"
            echo "  espflash flash --monitor target/riscv32imc-unknown-none-elf/release/arcade"
          '';
        };
      }
    );
}
