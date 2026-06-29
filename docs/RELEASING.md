# Releasing dosui

This is the checklist for cutting a release. Releases are tag-driven: pushing a
`v*` tag runs the [`Release`](../.github/workflows/release.yml) workflow, which
builds the bundled AppImage and attaches it (plus `SHA256SUMS`) to the GitHub
Release.

## Versioning

dosui follows [Semantic Versioning](https://semver.org/). While pre-1.0, minor
bumps may include breaking changes.

## Steps

1. **Make sure CI is green** on `main`.
2. **Bump the version** in [`Cargo.toml`](../Cargo.toml) (`version = "X.Y.Z"`),
   then run `cargo build` so `Cargo.lock` updates too.
3. **Update [`CHANGELOG.md`](../CHANGELOG.md):** move the `Unreleased` entries
   under a new `## [X.Y.Z] - YYYY-MM-DD` heading and refresh the compare links at
   the bottom.
4. **Add a `<release>`** entry to
   [`data/io.github.dosui.metainfo.xml`](../data/io.github.dosui.metainfo.xml)
   with the version and date.
5. **Commit** the bump:
   ```sh
   git commit -am "chore(release): vX.Y.Z"
   ```
6. **Tag and push:**
   ```sh
   git tag -a vX.Y.Z -m "dosui X.Y.Z"
   git push origin main --tags
   ```
7. The **Release workflow** builds `dosui-x86_64.AppImage`, generates
   `SHA256SUMS`, and creates the GitHub Release with auto-generated notes. Edit
   the release notes to lead with the changelog highlights if you like.

## What the workflow does

- Installs the GTK 4 dev libraries and a stable Rust toolchain.
- Runs [`packaging/build-appimage.sh`](../packaging/build-appimage.sh), which
  downloads the pinned `dosbox-staging` (`DOSBOX_STAGING_VERSION`), does a
  `--release` build, assembles the AppDir, bundles the GTK runtime via
  `linuxdeploy-plugin-gtk`, neutralises the forced GTK theme (so the AppImage
  follows the host theme), and packages the AppImage.
- Generates `SHA256SUMS` and uploads everything to the Release.

## Bumping the bundled dosbox-staging

Update `DOSBOX_STAGING_VERSION` in **both**
[`packaging/build-appimage.sh`](../packaging/build-appimage.sh) (the default) and
[`.github/workflows/release.yml`](../.github/workflows/release.yml) (the workflow
env), then test a local `make appimage`.

## Verifying a release

```sh
sha256sum -c SHA256SUMS         # against the downloaded AppImage
chmod +x dosui-x86_64.AppImage
./dosui-x86_64.AppImage
```
