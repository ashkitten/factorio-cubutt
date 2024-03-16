{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { nixpkgs, crane, fenix, flake-utils, ... }:
    flake-utils.lib.eachSystem [ "x86_64-linux" ] (system: let
      pkgs = nixpkgs.legacyPackages.${system};

      toolchain = with fenix.packages.${system}; combine [
        minimal.rustc
        minimal.cargo
        targets.x86_64-unknown-linux-musl.latest.rust-std
        targets.x86_64-pc-windows-gnu.latest.rust-std
      ];

      craneLib = (crane.mkLib pkgs).overrideToolchain toolchain;

      base = {
        src = craneLib.cleanCargoSource (craneLib.path ./native);

        strictDeps = true;

        CARGO_BUILD_RUSTFLAGS = "-C target-feature=+crt-static";
      };

      cubutt-native-linux = craneLib.buildPackage (base // {
        CARGO_BUILD_TARGET = "x86_64-unknown-linux-musl";

        nativeBuildInputs = with pkgs; [
          pkg-config
        ];

        buildInputs = with pkgs.pkgsStatic; [
          udev
          dbus
          openssl
        ];
      });

      cubutt-native-windows = craneLib.buildPackage (base // {
        CARGO_BUILD_TARGET = "x86_64-pc-windows-gnu";

        depsBuildBuild = with pkgs; [
          pkgsCross.mingwW64.stdenv.cc
          pkgsCross.mingwW64.windows.pthreads
        ];
      });
    in
      {
        checks = {
          inherit cubutt-native-linux cubutt-native-windows;
        };
      
        packages = {
          inherit cubutt-native-linux cubutt-native-windows;
          default = cubutt-native-linux;
        };
      }
    );
}
