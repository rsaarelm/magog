# Model for NixOS packaging script.
#
# After https://eipi.xyz/blog/packaging-a-rust-project-for-nix/
# Run `nix-build` to build
#
# `src = ./.` grabs everything from local dir, needs to be used on a clean
# checkout of the repo. Do `git clean -fdx` to clean up all local generated
# stuff before running.
#
# The cargoSha256 will get invalidated whenever new commits are made, running
# nix-build will give you an error message that contains the correct hash,
# replace value with that, then re-run nix-build.

with import <nixpkgs> { };

rustPlatform.buildRustPackage rec {
  name = "magog-${version}";
  version = "0.1.0";

  # Currently set to build from local sources. Once there are actual official
  # releases, switch to pulling the latest release from the web site:
  #
  #src = fetchFromGitHub {
  #  owner = "rsaarelm";
  #  repo = "magog";
  #  rev = "${version}";
  #  sha256 = "0avdnnhr5yscp1832g2y72x8msnc536n18ywkg6xdqd2ihm8bpca";
  #};

  src = ./.;

  buildInputs = [ openssl pkgconfig zlib gcc cmake ];

  checkPhase = "";
  cargoSha256 = "sha256:0avdnnhr5yscp1832g2y72x8msnc536n18ywkg6xdqd2ihm8bpca";

  # Binary size optimization
  prePatch = ''
    cat >> Cargo.toml << EOF
    [profile.release]
    lto = true
    codegen-units = 1
    panic = 'abort'
    EOF
  '';

  meta = with stdenv.lib; {
    description = "A fantasy roguelike game";
    homepage = https://github.com/rsaarelm/magog;
    license = licenses.agpl3;
    platforms = platforms.all;
  };
}
