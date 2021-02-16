{ common }:
with common; with pkgs;
devshell.mkShell {
  packages = [ rustc ];
  commands =
    let
      pkgc = pkg: { package = pkg; };
    in
    [
      (pkgc git)
      (pkgc nixpkgs-fmt)
    ];
}
