const KEYS = Object.keys(DEFAULTS);

async function load() {
  const stored = await browser.storage.local.get(KEYS);
  for (const key of KEYS) {
    const el = document.getElementById(key);
    if (!el) continue;
    const v = stored[key] ?? DEFAULTS[key];
    if (el.type === "checkbox") el.checked = Boolean(v);
    else el.value = v;
  }
}

function collect() {
  const out = {};
  for (const key of KEYS) {
    const el = document.getElementById(key);
    if (!el) continue;
    if (el.type === "checkbox") out[key] = el.checked;
    else if (el.type === "number") out[key] = Number(el.value);
    else out[key] = el.value;
  }
  return out;
}

function flash(msg, color) {
  const s = document.getElementById("status");
  s.textContent = msg;
  s.style.color = color || "#2a7";
  clearTimeout(flash._t);
  flash._t = setTimeout(() => { s.textContent = ""; }, 1600);
}

document.getElementById("save").addEventListener("click", async () => {
  await browser.storage.local.set(collect());
  flash("Saved");
});

document.getElementById("reset").addEventListener("click", async () => {
  await browser.storage.local.clear();
  await load();
  flash("Restored defaults");
});

load();
