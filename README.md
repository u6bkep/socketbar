# Socketbar

A Firefox extension that shows you every TCP port listening on your machine — in
the URL bar and in a toolbar popup. Click a port to open `http://localhost:PORT/`.

Linux only for now. The browser extension talks to a small native host (Rust,
~400 KB, no dynamic dependencies other than `libc`) that reads `/proc/net/tcp`
directly — no `ss`, `lsof`, or Python required.

## Why

Firefox's URL bar doesn't suggest the localhost ports you actually have open.
This fills the gap: list what's listening, filter out the noise, one click to
open it.

## Components

- `extension/` — Firefox WebExtension (manifest v2). Omnibox keyword `lh`,
  toolbar popup with filter, options page for deny/allow lists.
- `host/` — `socketbar-host` Rust binary. Speaks Firefox's native-messaging
  protocol (4-byte length prefix + JSON over stdio).
- `host-manifest/` — template for the native messaging host manifest that
  Firefox reads from `~/.mozilla/native-messaging-hosts/`.

## Install

All paths need Firefox 109+, and the Rust paths need `rustup` /
[rustup.rs](https://rustup.rs).

### With cargo, straight from git

```sh
cargo install --git https://github.com/u6bkep/socketbar socketbar-host
socketbar-host install
```

The first line builds and drops the binary into `~/.cargo/bin`. The second
writes `~/.mozilla/native-messaging-hosts/io.socketbar.host.json` pointing at
it, which is how Firefox finds the host. Re-run `socketbar-host install` after
each upgrade so the manifest points at the new binary.

Then load the extension (grab `socketbar-*.xpi` from a
[release](https://github.com/u6bkep/socketbar/releases/latest) and drag it into
`about:addons`, or clone this repo and load `extension/manifest.json` from
`about:debugging`).

To remove: `socketbar-host uninstall && cargo uninstall socketbar-host`.

### From a clone

```sh
git clone https://github.com/u6bkep/socketbar && cd socketbar
./install.sh
```

`install.sh` runs `cargo build --release` and `socketbar-host install` for you.
Then in Firefox:

1. `about:debugging#/runtime/this-firefox`
2. **Load Temporary Add-on…**
3. Pick `extension/manifest.json`
4. Click the toolbar icon, or type `lh` + space in the URL bar

### From a release binary (no Rust toolchain)

Download the matching `socketbar-host-*` binary from a
[release](https://github.com/u6bkep/socketbar/releases/latest), make it executable,
and run `./socketbar-host-<target> install` — the subcommand locates itself via
`current_exe()` and writes a manifest pointing at wherever you put the file.

For Firefox release builds, the `.xpi` must be signed by Mozilla — see
[AMO signing](https://extensionworkshop.com/documentation/publish/signing-and-distribution-overview/).
Until then, load it via `about:debugging` as above.

## Filters

The popup and omnibox hide listeners that usually aren't web servers. Open the
options page (`Settings` link in the popup) to edit:

- **Process deny/allow lists** — substring match against `/proc/<pid>/comm`
- **Port deny/allow lists** — single ports or ranges (`3000-3100`)
- **Loopback-only** — hide sockets that `localhost` can't reach
- **IPv4 + IPv6 dedupe** — collapse same-(port, process) duplicates
- **Random-high-port suppression** — hide high-numbered IPC ports from IDE /
  editor / browser processes

The popup has a **Show all** checkbox to bypass every filter temporarily.

## Native messaging protocol

The host reads/writes length-prefixed JSON on stdio. Each message is `u32`
(native byte order) length followed by a UTF-8 JSON body.

Requests:

```json
{ "id": "r1", "action": "list" }
```

Responses:

```json
{
  "id": "r1",
  "ports": [
    {
      "port": 8080,
      "addr": "127.0.0.1",
      "family": "v4",
      "process": "python3",
      "pid": 1234,
      "inode": 56789,
      "uid": 1000
    }
  ]
}
```

On error: `{ "id": "r1", "error": "…", "ports": [] }`.

## License

MIT — see [LICENSE](./LICENSE).
