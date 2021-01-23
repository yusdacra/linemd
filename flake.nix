{
  description = "Flake for linemd";

  inputs = {
    devshell.url = "github:numtide/devshell";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    naersk = {
      url = "github:nmattia/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flakeUtils.url = "github:numtide/flake-utils";
    nixpkgsMoz = {
      url = "github:mozilla/nixpkgs-mozilla";
      flake = false;
    };
  };

  outputs = inputs: with inputs;
    with flakeUtils.lib;
    eachSystem defaultSystems (system:
      let
        common = import ./nix/common.nix {
          sources = { inherit devshell naersk nixpkgs nixpkgsMoz; };
          inherit system;
        };

        packages = {
          # Compiles slower but has tests and faster executable
          "linemd" = import ./nix/build.nix {
            inherit common;
            doCheck = true;
            release = true;
          };
          # Compiles faster but no tests and slower executable
          "linemd-debug" = import ./nix/build.nix { inherit common; };
          # Compiles faster but has tests and slower executable
          "linemd-tests" = import ./nix/build.nix { inherit common; doCheck = true; };
        };
        apps = builtins.mapAttrs (n: v: mkApp { name = n; drv = v; exePath = "/bin/linemd"; }) packages;
      in
      {
        inherit packages apps;

        # Release build is the default package
        defaultPackage = packages."linemd";


        # Release build is the default app
        defaultApp = apps."linemd";


        devShell = import ./nix/devShell.nix { inherit common; };
      }
    );
}
