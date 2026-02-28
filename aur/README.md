# AUR poof-bin package

<https://aur.archlinux.org/packages/poof-bin>

## Automatic Build

Generation, commit and push of `PKGBUILD` and `.SRCINFO` files is handled
automatically by [AURA](https://github.com/pirafrank/aura).

## Manual Build

In case it should ever be needed.

### Pre

```sh
pacman -Sy
pacman -S namcap
```

### Build

```sh
# edit PKGBUILD file
nvim PKGBUILD
# generate
makepkg --printsrcinfo > .SRCINFO
# test
namcap PKGBUILD
# commit and push
git add . && git commit -S -m "Publish vX.Y.Z" && git push
```
