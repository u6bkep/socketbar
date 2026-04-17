use std::env;
use std::fs;
use std::io::{self, IsTerminal, Read, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use serde::{Deserialize, Serialize};

mod listeners;

const HOST_NAME: &str = "io.socketbar.host";
const EXTENSION_ID: &str = "socketbar@gecko.network";
const HOST_DESCRIPTION: &str = "Socketbar — enumerates listening TCP sockets";

#[derive(Deserialize)]
struct Request {
    id: String,
    action: String,
}

#[derive(Serialize)]
struct Response {
    id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    ports: Vec<listeners::Listener>,
}

fn read_message<R: Read>(r: &mut R) -> io::Result<Option<Vec<u8>>> {
    let mut len_buf = [0u8; 4];
    match r.read_exact(&mut len_buf) {
        Ok(()) => {}
        Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(e) => return Err(e),
    }
    let len = u32::from_ne_bytes(len_buf) as usize;
    let mut buf = vec![0u8; len];
    r.read_exact(&mut buf)?;
    Ok(Some(buf))
}

fn write_message<W: Write>(w: &mut W, resp: &Response) -> io::Result<()> {
    let body = serde_json::to_vec(resp)?;
    let len = u32::try_from(body.len())
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "response too large"))?;
    w.write_all(&len.to_ne_bytes())?;
    w.write_all(&body)?;
    w.flush()
}

fn handle(req: Request) -> Response {
    match req.action.as_str() {
        "list" => match listeners::list_listeners() {
            Ok(ports) => Response {
                id: req.id,
                error: None,
                ports,
            },
            Err(e) => Response {
                id: req.id,
                error: Some(e.to_string()),
                ports: Vec::new(),
            },
        },
        other => Response {
            id: req.id,
            error: Some(format!("unknown action: {}", other)),
            ports: Vec::new(),
        },
    }
}

fn serve() -> io::Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut stdin = stdin.lock();
    let mut stdout = stdout.lock();
    while let Some(buf) = read_message(&mut stdin)? {
        let req: Request = match serde_json::from_slice(&buf) {
            Ok(r) => r,
            Err(e) => {
                let resp = Response {
                    id: String::new(),
                    error: Some(format!("bad request: {}", e)),
                    ports: Vec::new(),
                };
                write_message(&mut stdout, &resp)?;
                continue;
            }
        };
        let resp = handle(req);
        write_message(&mut stdout, &resp)?;
    }
    Ok(())
}

fn manifest_path() -> io::Result<PathBuf> {
    let home = env::var_os("HOME")
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "$HOME not set"))?;
    Ok(PathBuf::from(home)
        .join(".mozilla/native-messaging-hosts")
        .join(format!("{HOST_NAME}.json")))
}

fn manifest_body(exe_path: &str) -> String {
    let body = serde_json::json!({
        "name": HOST_NAME,
        "description": HOST_DESCRIPTION,
        "path": exe_path,
        "type": "stdio",
        "allowed_extensions": [EXTENSION_ID],
    });
    serde_json::to_string_pretty(&body).expect("serializing static JSON can't fail")
}

fn install() -> io::Result<()> {
    let exe = env::current_exe()?;
    let exe_str = exe.to_str().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "executable path is not valid UTF-8",
        )
    })?;
    let manifest = manifest_path()?;
    if let Some(parent) = manifest.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut body = manifest_body(exe_str);
    body.push('\n');
    fs::write(&manifest, body)?;
    println!("Installed Firefox native-messaging host:");
    println!("  manifest: {}", manifest.display());
    println!("  binary:   {}", exe.display());
    println!();
    println!("Next: load the Socketbar extension in Firefox.");
    println!("  - Development:  about:debugging → Load Temporary Add-on → extension/manifest.json");
    println!("  - Release .xpi: drag into about:addons (must be Mozilla-signed)");
    Ok(())
}

fn uninstall() -> io::Result<()> {
    let manifest = manifest_path()?;
    match fs::remove_file(&manifest) {
        Ok(()) => println!("Removed {}", manifest.display()),
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            println!("Not installed (no file at {})", manifest.display());
        }
        Err(e) => return Err(e),
    }
    Ok(())
}

fn usage(out: &mut dyn Write) -> io::Result<()> {
    writeln!(out, "Socketbar native-messaging host")
        .and_then(|_| writeln!(out))
        .and_then(|_| writeln!(out, "USAGE:"))
        .and_then(|_| {
            writeln!(
                out,
                "  socketbar-host             run as a Firefox native-messaging host (no args)"
            )
        })
        .and_then(|_| {
            writeln!(
                out,
                "  socketbar-host install     write Firefox host manifest pointing at this binary"
            )
        })
        .and_then(|_| {
            writeln!(
                out,
                "  socketbar-host uninstall   remove the Firefox host manifest"
            )
        })
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        None => {
            if io::stdin().is_terminal() {
                let _ = usage(&mut io::stderr());
                return ExitCode::SUCCESS;
            }
            serve()
        }
        Some("install") => install(),
        Some("uninstall") => uninstall(),
        Some("-h" | "--help") => {
            let _ = usage(&mut io::stdout());
            return ExitCode::SUCCESS;
        }
        Some(other) => {
            eprintln!("unknown subcommand: {other}");
            let _ = usage(&mut io::stderr());
            return ExitCode::from(2);
        }
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_body_roundtrips() {
        let body = manifest_body("/home/example/.cargo/bin/socketbar-host");
        let v: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(v["name"], HOST_NAME);
        assert_eq!(v["type"], "stdio");
        assert_eq!(v["path"], "/home/example/.cargo/bin/socketbar-host");
        assert_eq!(v["allowed_extensions"], serde_json::json!([EXTENSION_ID]));
    }
}
