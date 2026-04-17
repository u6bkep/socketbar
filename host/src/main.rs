use std::io::{self, Read, Write};

use serde::{Deserialize, Serialize};

mod listeners;

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

fn main() -> io::Result<()> {
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
