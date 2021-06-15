let
  pkgs = import <nixpkgs> {};

  log_level = "info";
in
pkgs.mkShell {
  buildInputs = with pkgs; [
    rustc cargo rustfmt rust-analyzer cargo-outdated clippy

    # Needed by cargo dependencies.
    cmake gcc zlib pkgconfig openssl

    # wgpu graphics dependencies
    vulkan-loader vulkan-tools
    xorg.libXcursor xorg.libXi xorg.libXrandr

    # SPIR-V shader compiler
    shaderc

    # Linker
    lld

    # Map editor
    tiled
  ];

  shellHook = ''
    # Dynamic linking for Vulkan stuff for wgpu graphics
    export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${
      with pkgs; pkgs.lib.makeLibraryPath [ vulkan-loader openssl zlib ]
    }"

    # Run clippy without showing stuff I don't care about.
    alias clippy="cargo clippy -- -A clippy::cast_lossless"
  '';

  RUST_BACKTRACE = "1";
  RUSTFLAGS = "-C link-arg=-fuse-ld=lld";
  RUST_LOG = "calx-ecs=${log_level},vitral=${log_level},calx=${log_level},display=${log_level},world=${log_level},magog=${log_level}";
}
