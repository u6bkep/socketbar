function describeProcess(e) {
  if (e.process && e.pid) return `${e.process} (pid ${e.pid})`;
  if (e.process) return e.process;
  if (e.pid) return `pid ${e.pid}`;
  if (e.owner) return `owned by ${e.owner}`;
  if (e.uid !== undefined && e.uid !== null) return `uid ${e.uid}`;
  return "unknown";
}

let entries = [];
let rawCount = 0;
const $q = document.getElementById("q");
const $list = document.getElementById("list");
const $status = document.getElementById("status");
const $count = document.getElementById("count");
const $showAll = document.getElementById("showAll");
const $settings = document.getElementById("settings");

function render() {
  const q = $q.value.trim().toLowerCase();
  const filtered = entries.filter((e) => {
    if (!q) return true;
    return (
      String(e.port).includes(q) ||
      (e.process || "").toLowerCase().includes(q) ||
      (e.owner || "").toLowerCase().includes(q) ||
      (e.addr || "").toLowerCase().includes(q)
    );
  });
  $list.innerHTML = "";
  for (const e of filtered) {
    const li = document.createElement("li");
    li.dataset.port = e.port;

    const port = document.createElement("span");
    port.className = "port";
    port.textContent = `:${e.port}`;

    const proc = document.createElement("span");
    proc.className = "proc";
    proc.textContent = describeProcess(e);

    const addr = document.createElement("span");
    addr.className = "addr";
    addr.textContent = e.addr || "";

    li.append(port, proc, addr);
    li.addEventListener("click", (ev) => openEntry(e, ev));
    $list.appendChild(li);
  }
  if (filtered.length === 0) {
    $status.textContent = entries.length ? "No matches" : "No listening ports";
    $status.hidden = false;
  } else {
    $status.hidden = true;
  }
  const hidden = rawCount - entries.length;
  if ($showAll.checked) {
    $count.textContent = `${entries.length} listening`;
  } else if (hidden > 0) {
    $count.textContent = `${entries.length} shown · ${hidden} filtered`;
  } else {
    $count.textContent = `${entries.length} shown`;
  }
}

function openEntry(e, ev) {
  const url = `http://localhost:${e.port}/`;
  const background = ev.ctrlKey || ev.metaKey || ev.shiftKey;
  browser.tabs.create({ url, active: !background });
  if (!background) window.close();
}

async function load() {
  try {
    const response = await browser.runtime.sendMessage({ type: "list", showAll: $showAll.checked });
    if (response.error) throw new Error(response.error);
    entries = (response.ports || []).sort((a, b) => a.port - b.port);
    rawCount = response.raw ?? entries.length;
    $status.hidden = true;
    $status.classList.remove("error");
    render();
  } catch (err) {
    entries = [];
    rawCount = 0;
    $status.textContent = `Error: ${err.message || err}`;
    $status.classList.add("error");
    $status.hidden = false;
    $count.textContent = "";
  }
}

$q.addEventListener("input", render);
$q.addEventListener("keydown", (ev) => {
  if (ev.key === "Enter") {
    const first = $list.querySelector("li");
    if (first) {
      const port = Number(first.dataset.port);
      const entry = entries.find((e) => e.port === port);
      if (entry) openEntry(entry, ev);
    }
  }
});

$showAll.addEventListener("change", load);
$settings.addEventListener("click", (ev) => {
  ev.preventDefault();
  browser.runtime.sendMessage({ type: "openOptions" });
  window.close();
});

load();
