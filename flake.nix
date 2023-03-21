{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-22.11";

    fenixrs = {
      url = "github:nix-community/fenix";
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
                  pkgs.alsaLib
                  pkgs.udev
                  pkgs.vulkan-loader
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

              #          bevy-specific deps (from https://github.com/bevyengine/bevy/blob/main/docs/linux_dependencies.md)
              pkgconfig
              udev
              alsaLib
              xlibsWrapper
              xorg.libXcursor
              xorg.libXrandr
              xorg.libXi
              vulkan-tools
              vulkan-headers
              vulkan-loader
              vulkan-validation-layers
            ];

            # This is your devenv configuration
            #            packages = [ pkgs.hello ];
            #
            #            enterShell = ''
            #              hello
            #            '';
            #            processes.run.exec = "hello";
          })
        ];
      };
    };
}
