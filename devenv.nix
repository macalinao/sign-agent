{
  pkgs,
  lib,
  ...
}:

{
  packages = with pkgs; [
    git
    nixfmt-rfc-style

    rustup
    cargo-expand
    cargo-nextest

    # For TypeScript package
    bun

    # Build dependencies
    pkg-config
    openssl.dev
  ];

  # OpenSSL paths for openssl-sys crate
  env = {
    OPENSSL_LIB_DIR = "${pkgs.openssl.out}/lib";
    OPENSSL_INCLUDE_DIR = "${pkgs.openssl.dev}/include";
  };
}
