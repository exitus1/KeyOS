# SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
# SPDX-License-Identifier: GPL-3.0-or-later
{
  self,
  system,
  pkgs,
}: let
  src = pkgs.stdenv.mkDerivation {
    name = "cosign2-src";
    src = self + "/imports/cosign2";
    installPhase = ''
      cp -r . $out
    '';
    outputHash = "sha256-H0Eb1fsPgd4Yu9qGiKW3SHRr/K1VOoNxGBWqKNm9I5I=";
    outputHashMode = "recursive";
  };
in {
  cosign2 = pkgs.rustPlatform.buildRustPackage {
    name = "cosign2";
    inherit src;
    cargoLock = {
      lockFile = src + "/Cargo.lock";
    };
  };
}
