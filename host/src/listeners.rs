use std::collections::HashMap;
use std::fs;
use std::io;

use serde::Serialize;

#[derive(Serialize)]
pub struct Listener {
    pub port: u16,
    pub addr: String,
    pub family: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub process: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pid: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inode: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uid: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
}

const TCP_STATE_LISTEN: &str = "0A";

struct Raw {
    local: String,
    uid: u32,
    inode: u64,
    family: &'static str,
}

fn parse_proc_net(path: &str, family: &'static str) -> io::Result<Vec<Raw>> {
    let content = fs::read_to_string(path)?;
    let mut out = Vec::new();
    for (i, line) in content.lines().enumerate() {
        if i == 0 {
            continue;
        }
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() < 10 || fields[3] != TCP_STATE_LISTEN {
            continue;
        }
        let Ok(uid) = fields[7].parse::<u32>() else {
            continue;
        };
        let Ok(inode) = fields[9].parse::<u64>() else {
            continue;
        };
        out.push(Raw {
            local: fields[1].to_string(),
            uid,
            inode,
            family,
        });
    }
    Ok(out)
}

fn parse_hex_u8(s: &str) -> Option<u8> {
    u8::from_str_radix(s, 16).ok()
}

fn parse_hex_addr_v4(hex: &str) -> Option<(String, u16)> {
    let (a, p) = hex.split_once(':')?;
    if a.len() != 8 || p.len() != 4 {
        return None;
    }
    let port = u16::from_str_radix(p, 16).ok()?;
    let b = [
        parse_hex_u8(&a[0..2])?,
        parse_hex_u8(&a[2..4])?,
        parse_hex_u8(&a[4..6])?,
        parse_hex_u8(&a[6..8])?,
    ];
    // /proc/net/tcp stores the 4 IP bytes as a native-order u32 printed as %08X.
    // On little-endian the string order is reversed vs network order.
    let ip = format!("{}.{}.{}.{}", b[3], b[2], b[1], b[0]);
    Some((ip, port))
}

fn parse_hex_addr_v6(hex: &str) -> Option<(String, u16)> {
    let (a, p) = hex.split_once(':')?;
    if a.len() != 32 || p.len() != 4 {
        return None;
    }
    let port = u16::from_str_radix(p, 16).ok()?;
    // Four u32 words in host byte order, each printed as %08X.
    let mut bytes = [0u8; 16];
    for word in 0..4 {
        for byte in 0..4 {
            let offset = word * 8 + (3 - byte) * 2;
            bytes[word * 4 + byte] = parse_hex_u8(&a[offset..offset + 2])?;
        }
    }
    Some((format_ipv6(&bytes), port))
}

fn format_ipv6(bytes: &[u8; 16]) -> String {
    let groups: [u16; 8] =
        std::array::from_fn(|i| ((bytes[i * 2] as u16) << 8) | bytes[i * 2 + 1] as u16);
    let mut best_start = 0usize;
    let mut best_len = 0usize;
    let mut cur_start = 0usize;
    let mut cur_len = 0usize;
    for (i, g) in groups.iter().enumerate() {
        if *g == 0 {
            if cur_len == 0 {
                cur_start = i;
            }
            cur_len += 1;
            if cur_len > best_len {
                best_start = cur_start;
                best_len = cur_len;
            }
        } else {
            cur_len = 0;
        }
    }
    if best_len < 2 {
        return groups
            .iter()
            .map(|g| format!("{:x}", g))
            .collect::<Vec<_>>()
            .join(":");
    }
    let before: Vec<String> = groups[..best_start]
        .iter()
        .map(|g| format!("{:x}", g))
        .collect();
    let after: Vec<String> = groups[best_start + best_len..]
        .iter()
        .map(|g| format!("{:x}", g))
        .collect();
    format!("{}::{}", before.join(":"), after.join(":"))
}

fn display_addr(ip: &str) -> String {
    match ip {
        "0.0.0.0" | "::" => "all interfaces".to_string(),
        _ => ip.to_string(),
    }
}

fn build_inode_pid_map() -> HashMap<u64, u32> {
    let mut m = HashMap::new();
    let Ok(proc_dir) = fs::read_dir("/proc") else {
        return m;
    };
    for entry in proc_dir.flatten() {
        let Some(pid) = entry
            .file_name()
            .to_str()
            .and_then(|s| s.parse::<u32>().ok())
        else {
            continue;
        };
        let fd_dir = entry.path().join("fd");
        let Ok(fds) = fs::read_dir(&fd_dir) else {
            continue;
        };
        for fd in fds.flatten() {
            let Ok(target) = fs::read_link(fd.path()) else {
                continue;
            };
            let s = target.to_string_lossy();
            if let Some(rest) = s.strip_prefix("socket:[") {
                if let Some(inode_str) = rest.strip_suffix(']') {
                    if let Ok(inode) = inode_str.parse::<u64>() {
                        m.insert(inode, pid);
                    }
                }
            }
        }
    }
    m
}

fn read_proc_comm(pid: u32) -> Option<String> {
    fs::read_to_string(format!("/proc/{}/comm", pid))
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn load_uid_map() -> HashMap<u32, String> {
    let mut m = HashMap::new();
    let Ok(content) = fs::read_to_string("/etc/passwd") else {
        return m;
    };
    for line in content.lines() {
        let mut parts = line.split(':');
        let (Some(name), Some(_), Some(uid_str)) = (parts.next(), parts.next(), parts.next())
        else {
            continue;
        };
        if let Ok(uid) = uid_str.parse::<u32>() {
            m.insert(uid, name.to_string());
        }
    }
    m
}

pub fn list_listeners() -> io::Result<Vec<Listener>> {
    let mut raw = parse_proc_net("/proc/net/tcp", "v4")?;
    if let Ok(v6) = parse_proc_net("/proc/net/tcp6", "v6") {
        raw.extend(v6);
    }

    let mut entries: Vec<Listener> = raw
        .into_iter()
        .filter_map(|r| {
            let (ip, port) = match r.family {
                "v4" => parse_hex_addr_v4(&r.local)?,
                _ => parse_hex_addr_v6(&r.local)?,
            };
            Some(Listener {
                port,
                addr: display_addr(&ip),
                family: r.family,
                process: None,
                pid: None,
                inode: Some(r.inode),
                uid: Some(r.uid),
                owner: None,
            })
        })
        .collect();

    let inode_pid = build_inode_pid_map();
    let uid_map = load_uid_map();

    for e in entries.iter_mut() {
        if let Some(inode) = e.inode {
            if let Some(&pid) = inode_pid.get(&inode) {
                e.pid = Some(pid);
                e.process = read_proc_comm(pid);
            }
        }
        if e.pid.is_none() {
            if let Some(uid) = e.uid {
                if let Some(name) = uid_map.get(&uid) {
                    e.owner = Some(name.clone());
                }
            }
        }
    }

    entries.sort_by_key(|e| e.port);
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ipv4_parse() {
        assert_eq!(
            parse_hex_addr_v4("0100007F:1F90"),
            Some(("127.0.0.1".into(), 8080))
        );
        assert_eq!(
            parse_hex_addr_v4("00000000:01BB"),
            Some(("0.0.0.0".into(), 443))
        );
    }

    #[test]
    fn ipv6_parse() {
        assert_eq!(
            parse_hex_addr_v6("00000000000000000000000001000000:1F90"),
            Some(("::1".into(), 8080))
        );
        assert_eq!(
            parse_hex_addr_v6("00000000000000000000000000000000:1F90"),
            Some(("::".into(), 8080))
        );
    }

    #[test]
    fn ipv6_format() {
        let mut b = [0u8; 16];
        b[15] = 1;
        assert_eq!(format_ipv6(&b), "::1");
        assert_eq!(format_ipv6(&[0u8; 16]), "::");
        b = [0u8; 16];
        b[0] = 0xfe;
        b[1] = 0x80;
        b[15] = 1;
        assert_eq!(format_ipv6(&b), "fe80::1");
    }

    #[test]
    fn display_addr_wildcards() {
        assert_eq!(display_addr("0.0.0.0"), "all interfaces");
        assert_eq!(display_addr("::"), "all interfaces");
        assert_eq!(display_addr("127.0.0.1"), "127.0.0.1");
    }
}
