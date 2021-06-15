let
  rpath = with pkgs; pkgs.lib.makeLibraryPath [ vulkan-loader ];

  # Tell Nix to ignore in-tree build artifact directory 'target' when
  # determining the unique source fingerprint of the repo.
  src = builtins.filterSource
    (path: type: type != "directory" || builtins.baseNameOf path != "target")
    ./.;

  pkgs = import <nixpkgs> {};
  sources = import ./nix/sources.nix;
  naersk = pkgs.callPackage sources.naersk {};

  version = "0.1.0";
  basename = "magog";

in naersk.buildPackage {
  inherit src;

  name = "${basename}-${version}";

  buildInputs = with pkgs; [
    # Needed by cargo dependencies.
    cmake gcc zlib pkgconfig openssl

    # wgpu graphics dependencies
    vulkan-loader xorg.libXcursor xorg.libXi xorg.libXrandr

    makeWrapper
  ];

  # Optimize binary size.
  prePatch = ''
    cat >> Cargo.toml << EOF

    [profile.release]
    lto = true
    codegen-units = 1
    panic = 'abort'
    EOF
  '';

  # Wrap with necessary runtime libraries
  postInstall = ''
    if [[ -f $out/bin/${basename} ]]; then
      wrapProgram $out/bin/${basename} --prefix LD_LIBRARY_PATH : "${rpath}"
    fi
  '';

  meta = with pkgs.lib; {
    description = "A fantasy roguelike game";
    homepage = https://github.com/rsaarelm/magog;
    license = licenses.agpl3;
    platforms = platforms.all;
  };
}
