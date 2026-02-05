#!/usr/bin/env python3
"""
Update Homebrew formula from GitHub releases.

This script fetches the latest release from GitHub and extracts asset
SHA256 checksums directly from the API response, then renders the
Homebrew formula template.
"""

import argparse
import sys
from pathlib import Path
from typing import Dict, Optional

import requests
from minijinja import Environment

def fetch_latest_release(github_repo: str) -> Dict:
    """Fetch the latest release information from GitHub API."""
    url = f"https://api.github.com/repos/{github_repo}/releases/latest"
    print(f"Fetching latest release from {url}...")

    try:
        response = requests.get(url, timeout=30)
        response.raise_for_status()
        return response.json()
    except requests.exceptions.RequestException as e:
        print(f"Error fetching release: {e}", file=sys.stderr)
        sys.exit(1)


def find_asset(assets: list, pattern: str) -> Optional[Dict]:
    """Find an asset by filename pattern."""
    for asset in assets:
        if asset["name"] == pattern:
            return asset
    return None


def extract_version(tag_name: str) -> str:
    """Extract version from tag name (removes 'v' prefix if present)."""
    return tag_name.lstrip("v")


def extract_sha256_from_digest(digest: str) -> str:
    """Extract SHA256 hash from GitHub digest field.

    Args:
        digest: Digest string in format "sha256:hash"

    Returns:
        The SHA256 hash string

    Raises:
        SystemExit: If digest format is invalid
    """
    if not digest or not digest.startswith("sha256:"):
        print(f"Error: Invalid digest format: {digest}", file=sys.stderr)
        sys.exit(1)

    return digest.split(":", 1)[1]


def main():
    """Main function to update AUR PKGBUILD."""

    github_repo = "pirafrank/poof"
    template_path = Path(__file__).parent.resolve() / "PKGBUILD.jinja"
    output_path = Path(__file__).parent.resolve() / "PKGBUILD"

    # Fetch latest release
    release = fetch_latest_release(github_repo)
    tag_name = release["tag_name"]
    version = extract_version(tag_name)
    assets = release["assets"]

    print(f"Latest release: {tag_name} (version: {version})")
    print(f"Found {len(assets)} assets")

    # Replace placeholders in asset patterns
    resolved_patterns = {
        "linux_x86_64": f"poof-{version}-x86_64-unknown-linux-gnu.tar.gz",
        "linux_aarch64": f"poof-{version}-aarch64-unknown-linux-gnu.tar.gz",
        "linux_armv7": f"poof-{version}-armv7-unknown-linux-gnueabihf.tar.gz",
        "linux_riscv64gc": f"poof-{version}-riscv64gc-unknown-linux-gnu.tar.gz",
    }

    # Find all required assets and extract checksums
    template_vars = {"version": version}

    for key, pattern in resolved_patterns.items():
        asset = find_asset(assets, pattern)
        if not asset:
            print(f"Error: Required asset '{pattern}' not found in release", file=sys.stderr)
            sys.exit(1)

        # Extract SHA256 from digest field
        digest = asset.get("digest")
        if not digest:
            print(f"Error: No digest found for asset '{pattern}'", file=sys.stderr)
            print(f"Asset data: {asset.get('name')}", file=sys.stderr)
            sys.exit(1)

        # Parse digest (format: "sha256:hash")
        sha256 = extract_sha256_from_digest(digest)

        template_vars[f"{key}_sha256"] = sha256
        print(f"Found asset: {asset['name']} (sha256: {sha256[:16]}...)")

    # Load and render template
    print(f"\nRendering template from {template_path}...")
    env = Environment()
    template_content = template_path.read_text()
    rendered = env.render_str(template_content, **template_vars)

    # Write output
    output_path.write_text(rendered)
    print(f"Successfully updated {output_path}")
    print(f"\nFormula updated to version {version}")


if __name__ == "__main__":
    main()
