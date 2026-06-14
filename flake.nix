{
  description = "Rust package";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
  };

  outputs =
    { self, nixpkgs }:
    let
      pkgs = nixpkgs.legacyPackages."x86_64-linux";
      texEnv = pkgs.texlive.combine {
        inherit (pkgs.texlive)
          scheme-basic
          collection-fontsrecommended
          collection-latex
          collection-latexextra
          collection-latexrecommended
          algorithms
          algorithmicx
          ;
      };
      pythonEnv = pkgs.python3.withPackages (
        ps: with ps; [
          matplotlib
          numpy
          pandas
          polars
          pyarrow
          seaborn
        ]
      );
    in
    {

      devShells."x86_64-linux".default = pkgs.mkShell {
        buildInputs = with pkgs; [
          gcc
          rustc
          cargo
          rustfmt
          clippy
          rust-analyzer
          gnumake
          pythonEnv
          texEnv
          manim
        ];
        env.RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
      };

    };
}
