use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use bytes::BytesMut;
use std::sync::Arc;

use crate::resp;
use crate::db;

pub async fn handle_connection(mut socket: TcpStream, db: Arc<db::DB>) {
    eprintln!("Handling new connection");
    let mut buf = BytesMut::with_capacity(1024);

    loop {
        let mut temp_buf = [0; 1024];

        let n = match socket.read(&mut temp_buf).await {
            Ok(0) => { eprintln!("Connection closed"); return; }
            Ok(n) => { eprintln!("Read {} bytes", n); n }
            Err(_) => { eprintln!("Read error"); return; }
        };

        buf.extend_from_slice(&temp_buf[..n]);

        while let Some(value) = resp::parse_resp(&mut buf) {
            eprintln!("Parsed command: {:?}", value);
            let response = dispatch_command(value, &db);
            eprintln!("Response: {:?}", String::from_utf8_lossy(&response));
            if let Err(_) = socket.write_all(&response).await {
                return;
            }
        }
    }
}

pub fn dispatch_command(cmd: resp::Value, db: &db::DB) -> Vec<u8> {
    match cmd {
        resp::Value::Array(arr) if !arr.is_empty() => {
            if let resp::Value::BulkString(cmd_name) = &arr[0] {
                let cmd_str = std::str::from_utf8(&cmd_name).unwrap_or("").to_uppercase();
                match cmd_str.as_str() {
                    "PING" => b"+PONG\r\n".to_vec(),
                    "GET" if arr.len() == 2 => {
                        if let resp::Value::BulkString(key) = &arr[1] {
                            let key_str = std::str::from_utf8(&key).unwrap_or("");
                            if let Some(value) = db.get(key_str) {
                                let len = value.len();
                                let mut resp = format!("${}\r\n", len).into_bytes();
                                resp.extend_from_slice(&value);
                                resp.extend_from_slice(b"\r\n");
                                resp
                            } else {
                                b"$-1\r\n".to_vec()
                            }
                        } else {
                            b"-ERR wrong number of arguments for 'get' command\r\n".to_vec()
                        }
                    }
                    "SET" if arr.len() == 3 => {
                        if let (resp::Value::BulkString(key), resp::Value::BulkString(value)) = (&arr[1], &arr[2]) {
                            let key_str = String::from_utf8_lossy(&key).to_string();
                            db.set(key_str, value.clone());
                            b"+OK\r\n".to_vec()
                        } else {
                            b"-ERR wrong type of arguments for 'set' command\r\n".to_vec()
                        }
                    }
                    "SIMILARITY" if arr.len() == 3 => {
                        if let (resp::Value::BulkString(key), resp::Value::BulkString(k_str)) = (&arr[1], &arr[2]) {
                            let key_str = std::str::from_utf8(&key).unwrap_or("");
                            let k: usize = match std::str::from_utf8(&k_str).unwrap_or("").parse() {
                                Ok(num) => num,
                                Err(_) => return b"-ERR invalid k value\r\n".to_vec(),
                            };
                            if let Some(query_vec) = db.get_vector(key_str) {
                                let similar_keys = db.find_similar(&query_vec, k);
                                let mut resp = format!("*{}\r\n", similar_keys.len()).into_bytes();
                                for skey in similar_keys {
                                    resp.extend_from_slice(format!("${}\r\n", skey.len()).as_bytes());
                                    resp.extend_from_slice(skey.as_bytes());
                                    resp.extend_from_slice(b"\r\n");
                                }
                                resp
                            } else {
                                b"-ERR key not found or no embedding\r\n".to_vec()
                            }
                        } else {
                            b"-ERR wrong type of arguments for 'similarity' command\r\n".to_vec()
                        }
                    }
                    _ => b"-ERR unknown command\r\n".to_vec(),
                }
            } else {
                b"-ERR command must be bulk string\r\n".to_vec()
            }
        }
        _ => b"-ERR command must be array\r\n".to_vec(),
    }
}