{
  description = "Flake for linemd";

  inputs = {
    nixCargoIntegration.url = "github:yusdacra/nix-cargo-integration";
    flakeCompat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
  };

  outputs = inputs: inputs.nixCargoIntegration.lib.makeOutputs {
    root = ./.;
    buildPlatform = "naersk";
    overrides = {
      build = _: prev: {
        cargoTestOptions = def: (prev.cargoTestOptions def) ++ [ "--features" "svg" ];
      };
    };
  };
}
