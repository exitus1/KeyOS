# SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
# SPDX-License-Identifier: GPL-3.0-or-later
{
  description = "KeyOS development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    fenix,
  }: let
    inherit (nixpkgs) lib;

    systems = [
      "aarch64-darwin"
      "x86_64-darwin"
      "aarch64-linux"
      "x86_64-linux"
    ];

    forAllSystems = f:
      lib.foldl' lib.recursiveUpdate {} (
        map (
          system:
            lib.mapAttrs (_: value: {${system} = value;}) (f system)
        )
        systems
      );
  in
    forAllSystems (system: let
      pkgs = import nixpkgs {
        inherit system;
        config = {
          allowUnfree = true;
          permittedInsecurePackages = ["segger-jlink-qt4-810"];
          segger-jlink.acceptLicense = true;
        };
      };

      customPackages = let
        ciPkgs = with pkgs; {
          inherit just reuse taplo;
          # upstream slint-lsp for CI (faster)
          slint-lsp-upstream = slint-lsp;
        };
        rustToolchain = import ./nix/rust-toolchain.nix {
          inherit self system fenix;
          pkgs = pkgs;
        };
        slintPkgs = import ./nix/slint.nix {inherit self system pkgs;};
        cosign2Pkgs = import ./nix/cosign2.nix {inherit self system pkgs;};
        localazy = import ./nix/localazy.nix {inherit self system pkgs;};
      in
        ciPkgs // rustToolchain // slintPkgs // cosign2Pkgs // localazy;

      buildPackages = with pkgs;
        [
          bc
          taplo
          cmake
          curl
          gcc-arm-embedded
          git
          gnumake
          just
          openssl
          pkg-config
          reuse
          unixtools.xxd
        ]
        ++ (with customPackages; [
          cosign2
          rust-keyos
        ]);

      devPackages =
        buildPackages
        ++ (with customPackages; [
          localazy
          rust-analyzer
          slint-lsp
          slint-viewer
        ])
        ++ (
          with pkgs;
            [minicom]
            ++ lib.optionals stdenv.isLinux [
              segger-jlink
            ]
        );

      darwinPackages = let
        xcodeenv = import (nixpkgs + "/pkgs/development/mobile/xcodeenv") {inherit (pkgs) callPackage;};
      in
        lib.optionals pkgs.stdenv.isDarwin [
          (xcodeenv.composeXcodeWrapper {versions = ["16.0"];})
        ];

      linuxPackages = with pkgs;
        lib.optionals stdenv.isLinux [
          clang
          gcc
          llvmPackages.libclang
          llvmPackages.libcxxClang
          llvmPackages.llvm
          udev
        ];

      linuxAttrs = lib.optionalAttrs pkgs.stdenv.isLinux {
        # for bindgen in c++ libs
        # macos already has xcode clang
        LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
      };

      mkShell = packages:
        pkgs.mkShellNoCC (
          {
            strictDeps = true;
            packages = packages ++ linuxPackages ++ darwinPackages;
            hardeningDisable = ["all"];
            buildInputs = with pkgs;
              [
                pcsclite
              ]
              ++ lib.optionals stdenv.isLinux [
                udev
              ];

            LD_LIBRARY_PATH = with pkgs;
              lib.makeLibraryPath (
                [
                  fontconfig
                  pcsclite
                  # slint sim
                  libxkbcommon
                ]
                ++ lib.optionals stdenv.isLinux [
                  udev
                  llvmPackages.libclang.lib
                  # slint sim
                  xorg.libX11
                  xorg.libXcursor
                  xorg.libXi
                  wayland
                ]
              );

            shellHook = ''
              # darwin xcode
              unset DEVELOPER_DIR
              unset SDKROOT

              # unset clang env variables
              unset CC
              unset CXX
              unset AR
              unset RANLIB
            '';
          }
          // linuxAttrs
        );
    in {
      packages = customPackages;
      formatter = pkgs.alejandra;
      devShells = {
        # full development shell
        default = mkShell devPackages;
        # minimal build shell
        build = mkShell buildPackages;
      };
    });
}
