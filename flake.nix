{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    flake-utils.url = "github:numtide/flake-utils";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
  };

  outputs = { nixpkgs, crane, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachSystem [ "x86_64-linux" ] (system: let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [ (import rust-overlay) ];
      };

      rustToolchain = pkgs.rust-bin.stable.latest.default.override {
        targets = [ "x86_64-unknown-linux-musl" ];
      };

      craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

      cubutt-native = craneLib.buildPackage {
        src = craneLib.cleanCargoSource (craneLib.path ./native);

        strictDeps = true;

        CARGO_BUILD_TARGET = "x86_64-unknown-linux-musl";
        CARGO_BUILD_RUSTFLAGS = "-C target-feature=+crt-static";

        nativeBuildInputs = with pkgs; [
          pkg-config
        ];

        buildInputs = with pkgs.pkgsStatic; [
          udev
          dbus
          openssl
        ];
      };
    in
      {
        checks = {
          inherit cubutt-native;
        };

        packages.default = cubutt-native;
      }
    );
}
