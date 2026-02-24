# MacPorts

Scripts and GitHub Actions workflow for generating and submitting a MacPorts Portfile for poof.

## Prerequisites

- macOS with [MacPorts](https://www.macports.org/install.php) installed
- Rust toolchain (`rustup`)
- `git`
- [`gh` CLI](https://cli.github.com), authenticate with: `gh auth login`
- [`cargo2ports`](https://github.com/herbygillot/cargo2ports) installed

## Scripts

### `generate_portfile.sh`

Generates a MacPorts Portfile for a given version of poof.

```bash
# it auto-detects the latest version if no argument is provided
./macports/generate_portfile.sh
# or specify a version explicitly (without 'v' prefix), e.g.
./macports/generate_portfile.sh 1.2.3
```

If no version argument is given, the latest git tag is used automatically.

What it does:

1. Downloads the source tarball and `poof.1` man page from the GitHub release
2. Computes `rmd160`, `sha256`, and `size` checksums for each file
3. Generates the Cargo dependency block automatically by parsing `Cargo.lock`
4. Writes the Portfile to `$HOME/pirafrank/ports/sysutils/poof/Portfile`
5. Runs `portindex` to update the local MacPorts index

To test the generated port locally after running the script:

```bash
sudo port -v install poof
```

### `macport_pr.sh`

Submits the generated Portfile as a pull request to the upstream `macports/macports-ports` repository.

```bash
./macports/macport_pr.sh <portfile_path>
# e.g.
./macports/macport_pr.sh "$HOME/pirafrank/ports/sysutils/poof/Portfile"
```

What it does:

1. Validates the Portfile path and extracts the version from it
2. Prompts interactively for confirmation (prompt is skipped automatically in GitHub Actions)
3. Forks `macports/macports-ports` under your account if not already done
4. Creates a branch named `poof-<version>`, copies the Portfile, commits, and pushes
5. Opens a pull request against `macports/macports-ports`

## Test Locally

Before you can submit a PR, you should test the generated Portfile locally.

Pre-requisites:

Edit `/opt/local/etc/macports/sources.conf` to include the local ports directory at the top:

```conf
file:///Users/yourusername/pirafrank/ports [nosync]
# default MacPorts sources follows...
[default]
```

Then you can test the generated Portfile:

1. Run `generate_portfile.sh` with the desired version to create the Portfile in your local MacPorts tree.
2. Run `sudo port -v install poof` to test the installation of the new version.

## GitHub Actions Workflow

`.github/workflows/macports.yml` automates the full process on a macOS Intel runner.

### Triggers

The `release` trigger fires after the release is fully published, ensuring the source tarball and `poof.1` man page asset are already available for download.

### Token requirements

> **Important**: the default `GITHUB_TOKEN` provided by GitHub Actions **cannot fork repositories owned by other users or organizations**. The `gh repo fork macports/macports-ports` step will fail with a permission error if `GITHUB_TOKEN` is used.

A Personal Access Token (PAT) with `repo` scope is required and must be stored as a repository secret named `GH_PAT`.

#### 1. Create a Personal Access Token (classic)

1. Go to **GitHub → Settings → Developer settings → Personal access tokens → Tokens (classic)**
2. Click **Generate new token (classic)**
3. Give it a name (e.g. `poof-macports-pr`) and set an expiration
4. Select the **`repo`** scope (full control of public/private repositories — required for forking)
5. Click **Generate token** and copy the value immediately

#### 2. Store it as a repository secret

1. Go to your repository → **Settings → Secrets and variables → Actions**
2. Click **New repository secret**
3. Name: `GH_PAT`
4. Value: paste the token
5. Click **Add secret**

The workflow references this secret as `GH_TOKEN: ${{ secrets.GH_PAT }}`, which is what the `gh` CLI reads automatically.
