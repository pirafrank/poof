{
  description = "Easy to use zero-config, zero-install, zero-dependencies manager of pre-built software that works like magic";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      supportedSystems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];

      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
    in
    {
      packages = forAllSystems (system:
        let
          pkgs = nixpkgs.legacyPackages.${system};

          # Use the flake revision when available (e.g. `nix profile install github:pirafrank/poof`).
          # Falls back to "unknown" for local dirty builds (`nix build .#default`).
          gitCommitHash = if self ? rev then self.rev else "unknown";

          # Format lastModifiedDate (e.g. "20230615120000") to "YYYY-MM-DD HH:MM:SS UTC".
          # Falls back to the Unix epoch when not available.
          rawDate = if self ? lastModifiedDate then self.lastModifiedDate else "19700101000000";
          buildDate =
            "${builtins.substring 0 4 rawDate}-${builtins.substring 4 2 rawDate}-${builtins.substring 6 2 rawDate}"
            + " ${builtins.substring 8 2 rawDate}:${builtins.substring 10 2 rawDate}:${builtins.substring 12 2 rawDate} UTC";

          poof = pkgs.rustPlatform.buildRustPackage {
            pname = "poof";
            version = "0.6.0";

            # Filter out build artifacts and editor noise so the source hash stays stable.
            src = pkgs.lib.cleanSource ./.;

            cargoLock = {
              lockFile = ./Cargo.lock;
            };

            nativeBuildInputs = with pkgs; [
              pkg-config
            ];

            # System C libraries required by bzip2-sys (bzip2 crate) and lzma-sys (xz2 crate).
            buildInputs = with pkgs; [
              bzip2
              xz
            ];

            # Derivation-level env vars consumed by build.rs (Option B).
            # build.rs uses these when set and falls back to git/chrono otherwise,
            # so normal `cargo build` outside Nix is unaffected.
            env = {
              GIT_COMMIT_HASH = gitCommitHash;
              BUILD_DATE = buildDate;
            };

            # The Nix build sandbox sets HOME=/homeless-shelter (non-writable) and
            # leaves XDG_DATA_HOME unset. Tests that rely on dirs::data_dir() need a
            # writable home and a valid XDG_DATA_HOME before the test binary starts.
            preCheck = ''
              export HOME=$(mktemp -d)
              export XDG_DATA_HOME="$HOME/.local/share"
              mkdir -p "$XDG_DATA_HOME"
            '';

            meta = with pkgs.lib; {
              description = "Easy to use zero-config, zero-install, zero-dependencies manager of pre-built software that works like magic";
              homepage = "https://github.com/pirafrank/poof";
              license = licenses.mit;
              mainProgram = "poof";
              platforms = platforms.unix;
            };
          };
        in
        {
          default = poof;
          poof = poof;
        }
      );
    };
}
