function parsePortSet(text) {
  const tokens = String(text || "").split(/[\s,]+/).filter(Boolean);
  const singles = new Set();
  const ranges = [];
  for (const t of tokens) {
    const m = t.match(/^(\d+)-(\d+)$/);
    if (m) ranges.push([Number(m[1]), Number(m[2])]);
    else if (/^\d+$/.test(t)) singles.add(Number(t));
  }
  return {
    empty: singles.size === 0 && ranges.length === 0,
    has(port) {
      if (singles.has(port)) return true;
      for (const [lo, hi] of ranges) if (port >= lo && port <= hi) return true;
      return false;
    }
  };
}

function parseNameSet(text) {
  const names = String(text || "").split(/[\s,\n]+/).map((s) => s.trim().toLowerCase()).filter(Boolean);
  return {
    empty: names.length === 0,
    matches(name) {
      if (!name) return false;
      const lower = name.toLowerCase();
      return names.some((n) => lower.includes(n));
    }
  };
}

function isLoopback(addr) {
  if (!addr) return false;
  if (addr === "all interfaces") return true;
  if (addr.startsWith("127.")) return true;
  if (addr === "::1") return true;
  return false;
}

async function loadSettings() {
  const stored = await browser.storage.local.get(Object.keys(DEFAULTS));
  return { ...DEFAULTS, ...stored };
}

function applyFilters(ports, settings) {
  const procDeny = parseNameSet(settings.processDenylist);
  const procAllow = parseNameSet(settings.processAllowlist);
  const portDeny = parsePortSet(settings.portDenylist);
  const portAllow = parsePortSet(settings.portAllowlist);
  const noisyProc = parseNameSet(settings.randomHighPortProcesses);

  let result = ports.slice();

  if (settings.dedupeFamilies) {
    const seen = new Map();
    for (const e of result) {
      const key = `${e.port}|${e.process || ""}|${e.pid || ""}`;
      if (!seen.has(key)) seen.set(key, e);
    }
    result = [...seen.values()];
  }

  if (settings.loopbackOnly) {
    result = result.filter((e) => isLoopback(e.addr));
  }

  if (!procAllow.empty) {
    result = result.filter((e) => procAllow.matches(e.process));
  } else if (!procDeny.empty) {
    result = result.filter((e) => !procDeny.matches(e.process));
  }

  if (!portAllow.empty) {
    result = result.filter((e) => portAllow.has(e.port));
  } else if (!portDeny.empty) {
    result = result.filter((e) => !portDeny.has(e.port));
  }

  if (settings.suppressRandomHighPorts) {
    result = result.filter((e) => {
      if (e.port < settings.randomHighPortMin) return true;
      if (!portAllow.empty && portAllow.has(e.port)) return true;
      return !noisyProc.matches(e.process);
    });
  }

  return result;
}
