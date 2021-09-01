with import <nixpkgs>{};

stdenv.mkDerivation {
    name = "bevy-spicy-data";

    buildInputs = [
        clang
        pkgconfig
    ];
}