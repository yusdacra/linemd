{ release ? false
, doCheck ? false
, doDoc ? false
, common
,
}:
with common;
let
  meta = with pkgs.lib; {
    homepage = "https://github.com/yusdacra/linemd";
    license = licenses.mit;
  };

  package = with pkgs; naersk.buildPackage {
    root = ../.;
    cargoTestOptions = def: def ++ [ "--lib" "--tests" "--bins" "--examples" ];
    overrideMain = (prev: {
      inherit meta;
    });
    inherit release doCheck doDoc;
  };
in
package
