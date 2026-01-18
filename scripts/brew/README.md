# Poof Homebrew Formula

```sh
shasum -a 256 assets/* | jq -R 'split(" ")[0]' > checksums.json
```

then

```sh
minijinja-cli --vars checksums.json template.j2 > poof.rb
```
