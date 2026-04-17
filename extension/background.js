const HOST_NAME = "io.socketbar.host";

let nativePort = null;
const pending = new Map();

function connect() {
  if (nativePort) return nativePort;
  nativePort = browser.runtime.connectNative(HOST_NAME);
  nativePort.onMessage.addListener((msg) => {
    const resolver = pending.get(msg.id);
    if (resolver) {
      pending.delete(msg.id);
      resolver(msg);
    }
  });
  nativePort.onDisconnect.addListener((p) => {
    const err = p.error ? p.error.message : "disconnected";
    for (const [id, resolve] of pending) resolve({ id, error: err, ports: [] });
    pending.clear();
    nativePort = null;
  });
  return nativePort;
}

function request(action) {
  const port = connect();
  const id = Math.random().toString(36).slice(2) + Date.now().toString(36);
  return new Promise((resolve) => {
    pending.set(id, resolve);
    try {
      port.postMessage({ id, action });
    } catch (e) {
      pending.delete(id);
      resolve({ id, error: String(e), ports: [] });
    }
  });
}

function describeProcess(entry) {
  if (entry.process && entry.pid) return `${entry.process} (pid ${entry.pid})`;
  if (entry.process) return entry.process;
  if (entry.pid) return `pid ${entry.pid}`;
  if (entry.owner) return `owned by ${entry.owner}`;
  if (entry.uid !== undefined && entry.uid !== null) return `uid ${entry.uid}`;
  return "unknown";
}

function formatEntry(entry) {
  const url = `http://localhost:${entry.port}/`;
  const addr = entry.addr ? ` — ${entry.addr}` : "";
  return {
    content: url,
    description: `:${entry.port}  →  ${describeProcess(entry)}${addr}`
  };
}

async function fetchPorts(showAll) {
  const r = await request("list");
  if (r.error) return { error: r.error, ports: [], raw: 0, filtered: 0, showAll: !!showAll };
  const raw = r.ports || [];
  if (showAll) return { ports: raw, raw: raw.length, filtered: raw.length, showAll: true };
  const settings = await loadSettings();
  const ports = applyFilters(raw, settings);
  return { ports, raw: raw.length, filtered: ports.length, showAll: false };
}

function setDefaultHint(count) {
  const tail = typeof count === "number"
    ? `${count} listening (filtered) — type any character to show them, or click the toolbar icon`
    : "type any character to list listening ports, or click the toolbar icon";
  browser.omnibox.setDefaultSuggestion({ description: tail });
}
setDefaultHint();

browser.omnibox.onInputStarted.addListener(async () => {
  const r = await fetchPorts(false);
  if (!r.error) setDefaultHint(r.filtered);
});

browser.runtime.onMessage.addListener((msg) => {
  if (msg && msg.type === "list") return fetchPorts(msg.showAll);
  if (msg && msg.type === "openOptions") return browser.runtime.openOptionsPage();
});

browser.omnibox.onInputChanged.addListener(async (text, suggest) => {
  const response = await fetchPorts(false);
  if (response.error) {
    suggest([{
      content: "about:blank",
      description: `Socketbar error: ${response.error}`
    }]);
    return;
  }
  const q = text.trim().toLowerCase();
  const filtered = (response.ports || []).filter((entry) => {
    if (!q) return true;
    const hay = `${entry.port} ${entry.process || ""} ${entry.owner || ""} ${entry.addr || ""}`.toLowerCase();
    return hay.includes(q);
  });
  filtered.sort((a, b) => a.port - b.port);
  suggest(filtered.slice(0, 12).map(formatEntry));
});

browser.omnibox.onInputEntered.addListener((input, disposition) => {
  let url = input.trim();
  if (/^\d+$/.test(url)) url = `http://localhost:${url}/`;
  else if (!/^https?:\/\//i.test(url)) url = `http://${url}`;
  if (disposition === "currentTab") browser.tabs.update({ url });
  else browser.tabs.create({ url, active: disposition !== "newBackgroundTab" });
});
