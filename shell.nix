{
  _workspace ? import ./. {},
  pkgs ? _workspace.pkgs,
  libraries ? _workspace.libraries,
  rust-toolchain ? _workspace.rust-toolchain,
}:
let
  DATA_DIR = "${builtins.getEnv "HOME"}/.local/state/refoliate";
in pkgs.mkShell {
  inherit DATA_DIR;

  LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath libraries}";
  packages = with pkgs; [
    niv
    cargo-generate
    pkg-config
    rust-toolchain
  ] ++ libraries;
}

