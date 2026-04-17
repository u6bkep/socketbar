const DEFAULTS = {
  processDenylist: [
    "sshd", "cupsd", "avahi-daemon", "dnsmasq", "chronyd", "ntpd", "rpcbind",
    "postgres", "mysqld", "mariadbd", "mongod", "redis-server", "memcached",
    "smbd", "nmbd", "slapd", "saslauthd", "rsyslogd", "master", "opendkim",
    "opendmarc", "systemd-resolved", "systemd-timesyn", "systemd-udevd",
    "cups-browsed", "colord", "steam", "spotify", "kdeconnectd"
  ].join("\n"),
  processAllowlist: "",
  portDenylist: [
    "22", "25", "53", "67", "68", "69", "88", "110", "111", "123", "137", "138",
    "139", "143", "161", "162", "389", "443", "445", "465", "514", "515", "543",
    "544", "587", "631", "636", "873", "989", "990", "993", "995", "1080", "1194",
    "1433", "1521", "1701", "1812", "1813", "1900", "2049", "2082", "2083",
    "3306", "3389", "5060", "5061", "5222", "5269", "5353", "5432", "5900-5910",
    "6379", "6660-6669", "6697", "9050", "11211", "27017"
  ].join(" "),
  portAllowlist: "",
  loopbackOnly: true,
  dedupeFamilies: true,
  suppressRandomHighPorts: true,
  randomHighPortMin: 32768,
  randomHighPortProcesses: [
    "code", "codium", "code-oss", "firefox", "chrome", "chromium", "brave",
    "electron", "pylance", "webstorm", "idea", "pycharm", "rustrover",
    "goland", "clion", "jetbrains", "plasmashell", "kwin"
  ].join(" ")
};
