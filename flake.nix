{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  outputs = {
    self,
    nixpkgs,
    crane,
    rust-overlay,
    ...
  }: let
    system = "x86_64-linux";
    pkgs = import nixpkgs {
      inherit system;
      overlays = [(import rust-overlay)];
    };

    enzymeLib = pkgs.fetchzip {
      url = "https://ci-artifacts.rust-lang.org/rustc-builds/ec818fda361ca216eb186f5cf45131bd9c776bb4/enzyme-nightly-x86_64-unknown-linux-gnu.tar.xz";
      sha256 = "sha256-Rnrop44vzS+qmYNaRoMNNMFyAc3YsMnwdNGYMXpZ5VY=";
    };

    rustToolchain = pkgs.symlinkJoin {
      name = "rust-with-enzyme";
      paths = [
        (pkgs.rust-bin.nightly.latest.default.override {
          extensions = ["rust-analyzer" "clippy" "rustfmt" "rust-src"];
        })
      ];
      nativeBuildInputs = [pkgs.makeWrapper];
      postBuild = ''
        libdir=$out/lib/rustlib/x86_64-unknown-linux-gnu/lib
        cp ${enzymeLib}/enzyme-preview/lib/rustlib/x86_64-unknown-linux-gnu/lib/libEnzyme-22.so $libdir/

        wrapProgram $out/bin/rustc \
          --add-flags "--sysroot $out"

        wrapProgram $out/bin/clippy-driver \
          --set SYSROOT "$out"

        wrapProgram $out/bin/cargo-clippy \
          --set SYSROOT "$out"

        wrapProgram $out/bin/rust-analyzer \
          --set RUST_SRC_PATH "$out/lib/rustlib/src/rust/library" \
          --set RUSTC "$out/bin/rustc"
      '';
    };

    craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;
    src = craneLib.cleanCargoSource ./.;
    ipoptDev = pkgs.symlinkJoin {
      name = "ipopt-dev-with-coin-symlink";
      paths = [pkgs.ipopt.dev pkgs.ipopt.out];
      postBuild = ''
        ln -s coin-or $out/include/coin
      '';
    };

    commonArgs = {
      inherit src;
      strictDeps = true;
      nativeBuildInputs = with pkgs; [pkg-config cmake llvmPackages.libclang];
      buildInputs = [ipoptDev pkgs.openssl];
      CMAKE_INCLUDE_PATH = "${ipoptDev}/include";
      LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
    };

    cargoArtifacts = craneLib.buildDepsOnly commonArgs;

    wblna_optim = craneLib.buildPackage (commonArgs // {pname = "wblna_optim";});
  in {
    checks.${system} = {
      inherit wblna_optim;
      clippy = craneLib.cargoClippy (commonArgs // {inherit cargoArtifacts;});
      fmt = craneLib.cargoFmt {inherit src;};
      toml-fmt = craneLib.taploFmt {
        src = pkgs.lib.sources.sourceFilesBySuffices src [".toml"];
      };
    };

    packages.${system}.default = wblna_optim;

    apps.${system}.default = {
      type = "app";
      program = "${wblna_optim}/bin/wblna_optim";
    };

    devShells.${system}.default = craneLib.devShell {
      checks = self.checks.${system};
      CMAKE_INCLUDE_PATH = "${ipoptDev}/include";
      LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";

      # Force cargo to use wrapped binaries explicitly
      RUSTC = "${rustToolchain}/bin/rustc";
      RUSTFMT = "${rustToolchain}/bin/rustfmt";

      shellHook = ''
        export PATH=${rustToolchain}/bin:$PATH
      '';

      packages = with pkgs; [
        cargo-nextest
        cargo-expand
        cargo-flamegraph
        (python3.withPackages (ps: [ps.matplotlib ps.numpy]))
      ];
    };
  };
}
