with import <nixpkgs> {};

let
  log_level = "info";

# XXX: using 'gcc9Stdenv' to get lld linking. Replace with 'stdenv' once NixOS
# has gcc9 as the default version.
in gcc9Stdenv.mkDerivation {
  name = "rust-env";
  buildInputs = [
    rustup

    # Needed by cargo dependencies.
    cmake gcc zlib pkgconfig openssl

    # wgpu graphics dependencies
    vulkan-loader
    vulkan-tools
    xorg.libXcursor
    xorg.libXi
    xorg.libXrandr

    # Linker
    lld

    # SPIR-V shader compiler
    shaderc

    # Map editor
    tiled
  ];

  # XXX: This isn't the proper Nix way to do setup
  # TODO: Support cross-compilation to target x86_64-pc-windows-gnu
  shellHook = ''
    # Dynamic linking for Vulkan stuff for wgpu graphics
    export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${
      with pkgs.xlibs; lib.makeLibraryPath
      [
        libXcursor
        libXi
        libXrandr
        vulkan-loader
      ]
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

  # Useful backtraces
  RUST_BACKTRACE = 1;
  # Better linking
  RUSTFLAGS = "-C link-arg=-fuse-ld=lld";
  # Activate logging, but from local crates only.
  RUST_LOG = "calx-ecs=${log_level},vitral=${log_level},calx=${log_level},display=${log_level},world=${log_level},magog=${log_level}";

}
