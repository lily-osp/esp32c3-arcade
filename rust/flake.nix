{
  description = "ESP32-C3 Retro Space Shooter in Rust (no_std)";

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
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          targets = [ "riscv32imc-unknown-none-elf" ];
        };
      in
      {
        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            rustToolchain
            espflash
            pkg-config
          ];

          shellHook = ''
            export PS1="[esp32c3-rust:\w]\$ "
            echo "ESP32-C3 Rust Embedded Environment Ready (Target: riscv32imc-unknown-none-elf)"
          '';
        };
      });
}
