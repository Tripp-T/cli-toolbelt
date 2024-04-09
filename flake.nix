{
	inputs = {
		rust-overlay.url = "github:oxalica/rust-overlay";
		nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
		flake-utils.url = "github:numtide/flake-utils";
	};
	outputs = { self, nixpkgs, flake-utils, rust-overlay }: 
		flake-utils.lib.eachDefaultSystem
			(system: let
				overlays = [ (import rust-overlay) ];
				pkgs = import nixpkgs {
					inherit system overlays;
				};
				rust-toolchain = pkgs.rust-bin.stable.latest.default.override {
					extensions = [ "rust-src" "rustfmt" ];
				};
			in {
				devShells.default = pkgs.mkShell {
					buildInputs = with pkgs; [
						rust-toolchain
						pkg-config
						openssl
					];
					packages = with pkgs; [
						cargo-watch
					];
					RUST_BACKTRACE = 1;
				};
			});
}
