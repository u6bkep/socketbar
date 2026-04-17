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

## Install (development)

Requires `rustup` / `cargo` and Firefox 109+.

```sh
./install.sh
```

Then in Firefox:

1. `about:debugging#/runtime/this-firefox`
2. **Load Temporary Add-on…**
3. Pick `extension/manifest.json`
4. Click the toolbar icon, or type `lh` + space in the URL bar

## Install (from release)

Grab the matching `socketbar-host-*` binary and `socketbar-*.xpi` from the
[latest release](https://github.com/_/socketbar/releases/latest). Drop the
binary somewhere on your `$PATH`, edit the `path` field in
`host-manifest/io.socketbar.host.json.template` to point at it, and copy it to
`~/.mozilla/native-messaging-hosts/io.socketbar.host.json`.

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
