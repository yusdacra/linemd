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
    buildPlatform = "crate2nix";
  };
}
