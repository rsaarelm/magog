# Model for NixOS packaging script.
#
# After https://eipi.xyz/blog/packaging-a-rust-project-for-nix/
# Run `nix-build` to build
#
# The cargoSha256 will get invalidated whenever new commits are made, running
# nix-build will give you an error message that contains the correct hash,
# replace value with that, then re-run nix-build.
#
# Use `src = ./.` instead of `src = fetchFromGithub { ... }` to build from
# local sources. Having build artifacts (`target/` directory) in the local
# directory seems to mess up `nix-build`, so do a `git clean -fdx` or clone a
# fresh instance repository before trying this.

with import <nixpkgs> { };

rustPlatform.buildRustPackage rec {
  name = "magog-${version}";
  version = "0.1.0";

  src = fetchFromGitHub {
    owner = "rsaarelm";
    repo = "magog";

    # TODO: Start using release versions when we have them.
    # Now just using an arbitrary master commit from when this script was
    # being set up.
    rev = "fa716ed154937b641a22fdc0c6366b1a4ace279c";
    # rev = "${version}";

    sha256 = "0z7ddzpibs5cgk3ijd6zhcyngkb0bfp7zfv4ykw1b2yhm4pi8vf0";
  };

  buildInputs = [ openssl pkgconfig zlib gcc cmake ];

  checkPhase = "cargo test --all";
  cargoSha256 = "sha256:0z54m1xnzfhgh8b6cscdp8dm036g39lwnnmrlv4vcxrvbp3x0ndw";

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
