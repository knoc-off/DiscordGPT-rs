{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-22.11";

    fenixrs = {
      url = "github:nix-community/fenix/monthly";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    devenv.url = "github:cachix/devenv";
  };

  outputs = { self, fenixrs, nixpkgs, devenv, ... }@inputs:
    let
      appName = "discord_gpt";
      version = "0.1.0";

      system = "x86_64-linux";

      fenix = import fenixrs { inherit system; };
      pkgs = import nixpkgs { inherit system; };

      rustPkgs = with fenix; combine (with default; [
        cargo
        clippy-preview
        latest.rust-src
        rust-analyzer
        rust-std
        rustc
        rustfmt-preview
      ]);

      packages = with pkgs; [
        rustPkgs
        cargo-edit
        cargo-watch
        mold
        lld
        clang
        fuse
        pkgconfig
        openssl
      ];
    in
    {
      devShell.${system} = devenv.lib.mkShell {
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
            packages = packages;
          })
        ];
      };

      packages.${system}.app =
        let
          originalApp = (pkgs.makeRustPlatform {
            cargo = fenix.minimal.cargo;
            rustc = fenix.minimal.rustc;
          }).buildRustPackage {
            pname = appName;
            version = version;
            cargoLock.lockFile = ./Cargo.lock;
            src = ./.;

            nativeBuildInputs = packages;
            buildInputs = with pkgs; [
              openssl
            ];
          };
        in
        pkgs.runCommandNoCC "${originalApp.pname}-wrapped"
          {
            buildInputs = with pkgs; [ makeWrapper ];
          } ''

          makeWrapper ${originalApp}/bin/${appName} $out/bin/${appName} \
            --prefix LD_LIBRARY_PATH : ${pkgs.lib.makeLibraryPath [ pkgs.openssl ]}
        '';
    };
}
