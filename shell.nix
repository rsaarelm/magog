let
  pkgs = import <nixpkgs> {};

  log_level = "info";
in
pkgs.mkShell {
  buildInputs = with pkgs; [
    rustc cargo rustfmt rls cargo-outdated clippy

    # Needed by cargo dependencies.
    cmake gcc zlib pkgconfig openssl

    # wgpu graphics dependencies
    vulkan-loader vulkan-tools
    xorg.libXcursor xorg.libXi xorg.libXrandr

    # SPIR-V shader compiler
    shaderc

    # Map editor
    tiled
  ];

  shellHook = ''
    # Dynamic linking for Vulkan stuff for wgpu graphics
    export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${
      with pkgs; pkgs.stdenv.lib.makeLibraryPath [ vulkan-loader ]
    }"

    # Run clippy without showing stuff I don't care about.
    alias clippy="cargo clippy -- -A clippy::cast_lossless"

    # FIXME: Current (2020-04-18) NixOS cargo-outdated is broken, you have to
    # do this stupid thing. Remove alias when it's fixed.
    alias cargo-outdated="cargo-outdated outdated"
  '';

  RUST_BACKTRACE = "1";
  RUST_LOG = "calx-ecs=${log_level},vitral=${log_level},calx=${log_level},display=${log_level},world=${log_level},magog=${log_level}";
}
