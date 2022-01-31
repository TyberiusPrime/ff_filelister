{
  inputs = {
    utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nmattia/naersk";
    rust-overlay.url = "github:oxalica/rust-overlay";
    naersk.inputs.nixpkgs.follows = "nixpkgs";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
    rust-overlay.inputs.flake-utils.follows = "nixpkgs";
    nixpkgs = {
      url =
        "github:NixOS/nixpkgs?rev=7e9b0dff974c89e070da1ad85713ff3c20b0ca97"; # that's 21.05

    };

    #mozillapkgs = {
    #url = "github:mozilla/nixpkgs-mozilla";
    #flake = false;
    #};
  };

  outputs = { self, nixpkgs, utils, naersk, rust-overlay }:
    utils.lib.eachDefaultSystem (system:
      let
        #pkgs = nixpkgs.legacyPackages."${system}";

        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        rust = pkgs.rust-bin.stable."1.56.0".default.override {
          extensions = [ "rustfmt" "clippy" ];

          #          targets = [ "x86_64-unknown-linux-musl" ];
        };

        # Override the version used in naersk
        naersk-lib = naersk.lib."${system}".override {
          cargo = rust;
          rustc = rust;
        };
      in rec {
        # `nix build`
        packages.my-project = naersk-lib.buildPackage {
          pname = "ff_filelister";
          root = ./.;
          buildInput = [ pkgs.ripgrep pkgs.breakpointHook ];
          overrideMain = old: {
            postUnpack = ''
              substituteInPlace /build/source/src/main.rs --replace "/usr/bin/rg" "${pkgs.ripgrep}/bin/rg"
            '';
          };
        };
        defaultPackage = packages.my-project;

        # `nix run`
        apps.my-project = utils.lib.mkApp { drv = packages.my-project; };
        defaultApp = apps.my-project;

        # `nix develop`
        devShell = pkgs.mkShell {
          # supply the specific rust version
          nativeBuildInputs =
            [ rust pkgs.rust-analyzer pkgs.git pkgs.cargo-udeps ];
        };
      });
}
# {
