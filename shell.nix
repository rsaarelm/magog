with import <nixpkgs> {};

stdenv.mkDerivation {
  name = "rust-env";
  buildInputs = [
    rustup

    # Dev stuff cargo dependencies might need
    cmake gcc zlib pkgconfig openssl

    # Map editor
    tiled
  ];

  # XXX: This isn't the proper Nix way to do setup
  # TODO: Support cross-compilation to target x86_64-pc-windows-gnu
  shellHook = ''
    export RUST_BACKTRACE=1

    # Load the GL and X11 stuff the graphics app wants to link dynamically to
    export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${
      with pkgs.xlibs; lib.makeLibraryPath
      [ pkgs.libGL libX11 libXcursor libXxf86vm libXi libXrandr ]
    }"

    rustup install stable
    rustup default stable
    rustup update

    rustup component add rls-preview rust-analysis rust-src
    rustup component add rls-preview rust-analysis rust-src --toolchain nightly
    rustup component add rustfmt-preview clippy-preview --toolchain nightly

    NIX_ENFORCE_PURITY=0 cargo install cargo-outdated
  '';

  # Set Environment Variables
  RUST_BACKTRACE = 1;
}
