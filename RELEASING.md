# Releasing Socketbar

Releases are cut from tags. Pushing a `v*` tag triggers the `build-release`
workflow, which builds the host binaries, packages and AMO-signs the `.xpi`,
and creates a GitHub Release with those artifacts attached.

## Version bumps

Two files carry a version — keep them in sync (same semver):

| File                       | Field             |
| -------------------------- | ----------------- |
| `extension/manifest.json`  | `"version"`       |
| `host/Cargo.toml`          | `version`         |

Bump both, then regenerate the lockfile so `Cargo.lock` reflects the new
version:

```sh
cargo update --manifest-path host/Cargo.toml --package socketbar-host
```

Semver rule of thumb for this project:

- **patch** — bug fixes, filter tweaks, UI polish. No protocol change.
- **minor** — new extension features, new host actions, new settings. The
  native-messaging wire format must remain backwards-compatible: an older host
  must still respond to `{"action":"list"}` with a `{"ports":[...]}` shape, and
  an older extension must ignore unknown fields on `Listener`.
- **major** — breaking protocol or settings-storage change.

## Cutting a release

```sh
# 1. Bump versions + lockfile, commit on main.
$EDITOR extension/manifest.json host/Cargo.toml
cargo update --manifest-path host/Cargo.toml --package socketbar-host
git commit -am "release v0.1.1"

# 2. Tag and push.
git tag v0.1.1
git push origin main v0.1.1
```

The tag push fires `.github/workflows/build-release.yml`:

1. `lint` + `test` must pass.
2. `build-host` produces `socketbar-host-x86_64-unknown-linux-gnu` and
   `socketbar-host-x86_64-unknown-linux-musl`.
3. `package-extension` zips `extension/` into `socketbar-<version>.xpi`.
4. `sign-extension` uploads the `.xpi` to AMO's unlisted channel using
   `AMO_JWT_ISSUER` / `AMO_JWT_SECRET` repo secrets, then downloads the
   Mozilla-signed `.xpi`.
5. `release` creates a GitHub Release, preferring the signed `.xpi` and
   attaching the host binaries alongside.

The tag name (`v0.1.1`) and the version inside `manifest.json` should match —
the workflow reads the manifest version for the `.xpi` filename, but a drift
between the two just produces a confusing release page. Match them.

## Pre-flight checklist

Before tagging:

- `cargo fmt --all -- --check && cargo clippy --all-targets --release -- -D warnings` (run from `host/`)
- `cargo test --release` (from `host/`)
- Install locally (`./install.sh`) and click through the popup and options page
- Confirm the toolbar icon renders correctly

## AMO signing

The signing step needs two repo secrets, created at `addons.mozilla.org` →
_Developer Hub_ → _Manage API Keys_:

- `AMO_JWT_ISSUER`
- `AMO_JWT_SECRET`

Signing is on the **unlisted** channel — the `.xpi` is Mozilla-signed and
installable in stable Firefox, but isn't listed in the public catalog. If the
signing job fails (bad credentials, AMO outage), the release job falls back to
attaching the unsigned `.xpi`; stable Firefox users won't be able to install it
persistently until the next successful tag.

## Manual release (fallback)

If the workflow is wedged and you need to ship now:

```sh
# Host binaries
cargo build --release --manifest-path host/Cargo.toml --target x86_64-unknown-linux-gnu
cargo build --release --manifest-path host/Cargo.toml --target x86_64-unknown-linux-musl

# Extension
cd extension && zip -r -FS ../dist/socketbar-<version>.xpi . -x '*.DS_Store' && cd ..

# Sign (needs web-ext + AMO credentials in env)
cd extension && web-ext sign --channel=unlisted \
  --api-key="$AMO_JWT_ISSUER" --api-secret="$AMO_JWT_SECRET" \
  --artifacts-dir=../dist-signed && cd ..

# Upload via the GitHub UI or `gh release create v0.1.1 dist/* dist-signed/*`
```
