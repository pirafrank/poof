# AUR poof-bin package

<https://aur.archlinux.org/packages/poof-bin>

## Setup environment

### 1. Install Requirements

On Arch Linux:

```sh
pacman -Sy
pacman -S namcap
```

On Debian-based:

```sh
sudo apt-get update
sudo apt-get install -y pacman-package-manager
```

### 2. Setup virtualenv

```sh
python3 -m venv .venv
source .venv/bin/activate
pip install --upgrade pip
pip install -r requirements.txt
```

## Generate

```sh
source .venv/bin/activate
# generate PKGBUILD
python3 update.py
# generate .SRCINFO
makepkg --printsrcinfo > .SRCINFO
# test (Arch Linux-only)
namcap PKGBUILD
```
