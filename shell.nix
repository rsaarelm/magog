with import <nixpkgs> {};

let
  log_level = "info";
in stdenv.mkDerivation {
  name = "rust-env";
  buildInputs = [
    rustup

    # Needed by cargo dependencies.
    cmake gcc zlib pkgconfig openssl

    # Not needed for glium version, but if we upgrade to gfx-rs these are
    # needed to run Vulkan programs.
    x11 vulkan-loader vulkan-tools

    # X11 headers. Not currently used, but some Rust apps might want these.
    xorg.libXrandr
    xorg.libXinerama
    xorg.libXcursor
    xorg.libXi

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
      [ pkgs.libGL libX11 libXcursor libXxf86vm libXi libXrandr vulkan-loader ]
    }"

    rustup install stable
    rustup install nightly
    rustup default stable
    rustup update

    # Dev tools
    rustup component add clippy
    rustup component add rls-preview rust-analysis rust-src
    rustup component add rls-preview rust-analysis rust-src --toolchain nightly
    rustup component add rustfmt-preview --toolchain nightly

    # Show outdated crates
    NIX_ENFORCE_PURITY=0 cargo install cargo-outdated

    # Run clippy without showing stuff I don't care about.
    alias clippy="cargo clippy -- -A clippy::cast_lossless"
  '';

  # Set Environment Variables
  RUST_BACKTRACE = 1;

  RUST_LOG = "calx-ecs=${log_level},vitral=${log_level},calx=${log_level},display=${log_level},world=${log_level},magog=${log_level}";

}
