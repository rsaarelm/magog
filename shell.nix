with import <nixpkgs> {};

stdenv.mkDerivation {
  name = "rust-env";
  buildInputs = [
    rustup

    # Dev stuff cargo dependencies might need
    cmake gcc zlib pkgconfig openssl
  ];

  # XXX: This isn't the proper Nix way to do setup
  # TODO: Support cross-compilation to target x86_64-pc-windows-gnu
  shellHook = ''
    rustup install nightly
    rustup default nightly
    rustup update

    rustup component add rls-preview rust-analysis rust-src
    rustup component add rls-preview rust-analysis rust-src --toolchain nightly
    rustup component add rustfmt-preview clippy-preview --toolchain nightly

    # FIXME: These run into some linker problem when run from shellHook
    # They can be installed manually once in the shell though.
    # cargo install cargo-outdated
  '';

  # Set Environment Variables
  RUST_BACKTRACE = 1;
  RUST_LOG = "info";

  # Stuff needed to run Glium programs.
  LD_LIBRARY_PATH = with pkgs.xlibs;
     "${pkgs.mesa}/lib:"
    +"${libX11}/lib:"
    +"${libXcursor}/lib:"
    +"${libXxf86vm}/lib:"
    +"${libXi}/lib:"
    +"${libXrandr}/lib";
}
