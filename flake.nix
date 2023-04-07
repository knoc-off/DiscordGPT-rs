{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-22.11";

    fenixrs = {
      #url = "github:nix-community/fenix";
      url = "github:nix-community/fenix/monthly";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    devenv.url = "github:cachix/devenv";
  };

  outputs = { self, fenixrs, nixpkgs, devenv, ... }@inputs:
    let
      fenix = import fenixrs { system = "x86_64-linux"; };
      pkgs = import nixpkgs { system = "x86_64-linux"; };
    in {
      devShell.x86_64-linux = devenv.lib.mkShell {

        inherit inputs pkgs;
        modules = [
          ({ pkgs, ... }: {

            enterShell = ''
              export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${
                pkgs.lib.makeLibraryPath [
                  pkgs.openssl
                ]
              }"
            '';

            packages = with pkgs; [
              (with fenix;
                combine (with default; [
                  cargo
                  clippy-preview
                  latest.rust-src
                  rust-analyzer
                  rust-std
                  rustc
                  rustfmt-preview
                ]))
              cargo-edit
              cargo-watch
              mold
              lld
              clang
              fuse

              pkgconfig
              openssl
            ];
          })
        ];
      };
    };
}
