{ release ? false
, doCheck ? false
, doDoc ? false
, common
,
}:
with common;
let
  meta = with pkgs.stdenv.lib; {
    description = "Description for linemd";
    longDescription = ''Long description for linemd.'';
    homepage = "https://github.com/<owner>/linemd";
    license = licenses.mit;
  };



  package = with pkgs; naersk.buildPackage {
    root = ../.;
    nativeBuildInputs = crateDeps.nativeBuildInputs;
    buildInputs = crateDeps.buildInputs;
    override = (prev: env);
    overrideMain = (prev: {
      inherit meta;

    });

    inherit release doCheck doDoc;
  };
in
package
