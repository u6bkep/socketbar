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

Socketbar has two parts that install separately:

1. The **extension**, loaded by Firefox.
2. The **native host**, a binary Firefox launches to read `/proc`.

Both must be present for anything to work. Pick one option from each menu
below. Requires Firefox 109+.

### 1. Install the extension

<table>
<tr><th>Option</th><th>Gets you</th><th>Cost</th></tr>
<tr>
<td><b>Signed <code>.xpi</code> from a release</b><br>Download <code>socketbar-&lt;version&gt;.xpi</code> from the <a href="https://github.com/u6bkep/socketbar/releases/latest">latest release</a> and drag it into <code>about:addons</code>.</td>
<td>Persistent install. Works in any Firefox (stable, ESR, Developer, Nightly).</td>
<td>Requires a release tag that passed Mozilla's unlisted signing.</td>
</tr>
<tr>
<td><b>Temporary load from a clone</b><br>In Firefox, go to <code>about:debugging#/runtime/this-firefox</code> → <b>Load Temporary Add-on…</b> → pick <code>extension/manifest.json</code> from the repo.</td>
<td>Works off <code>main</code>, no signing needed. Good for development or trying an unreleased branch.</td>
<td>Disappears on every Firefox restart — re-load each session.</td>
</tr>
<tr>
<td><b>Unsigned <code>.xpi</code> in Developer Edition / Nightly</b><br>Set <code>xpinstall.signatures.required</code> to <code>false</code> in <code>about:config</code>, then drag the unsigned <code>.xpi</code> from CI artifacts or <code>zip</code> up <code>extension/</code> yourself.</td>
<td>Persistent install without waiting on AMO.</td>
<td>Only works in Developer Edition, Nightly, or ESR — not stable Firefox.</td>
</tr>
</table>

### 2. Install the native host

All three options finish by running `socketbar-host install`, which writes
`~/.mozilla/native-messaging-hosts/io.socketbar.host.json` pointing at the
binary. Firefox reads that file to locate the host.

<table>
<tr><th>Option</th><th>Gets you</th><th>Cost</th></tr>
<tr>
<td>

<b>cargo from git</b>

```sh
cargo install --git https://github.com/u6bkep/socketbar socketbar-host
socketbar-host install
```

</td>
<td>Latest <code>main</code>. Binary lives in <code>~/.cargo/bin</code>.</td>
<td>Needs <a href="https://rustup.rs">rustup</a>; compiles from source (~20 s).</td>
</tr>
<tr>
<td>

<b>Clone + <code>install.sh</code></b>

```sh
git clone https://github.com/u6bkep/socketbar && cd socketbar
./install.sh
```

</td>
<td>Same as above, but builds into <code>host/target/release</code> inside the clone.</td>
<td>Needs rustup. Binary stays in the repo — don't delete the clone.</td>
</tr>
<tr>
<td>

<b>Prebuilt binary from a release</b>

Download <code>socketbar-host-&lt;target&gt;</code> from the <a href="https://github.com/u6bkep/socketbar/releases/latest">latest release</a>, then:

```sh
chmod +x socketbar-host-*
./socketbar-host-* install
```

</td>
<td>No Rust toolchain required. The binary can live anywhere — <code>install</code> writes its absolute path into the manifest.</td>
<td>Linux x86_64 only. Move or delete the binary → re-run <code>install</code>.</td>
</tr>
</table>

After both parts are in place: click the Socketbar toolbar icon, or type `lh` +
space in the URL bar.

### Upgrading

Re-run `socketbar-host install` after every host upgrade (the manifest needs to
point at the new binary path). The extension auto-updates if you installed via
signed `.xpi` and it's enabled in `about:addons`; otherwise re-drag the new
`.xpi` or re-run `Load Temporary Add-on…`.

### Uninstall

```sh
socketbar-host uninstall      # removes the Firefox host manifest
cargo uninstall socketbar-host  # if installed via cargo
```

Remove the extension from `about:addons`.

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
