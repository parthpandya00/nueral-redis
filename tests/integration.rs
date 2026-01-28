use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::sync::Arc;
use std::time::Duration;

use neural_redis::db;
use neural_redis::connection;

#[tokio::test]
async fn test_e2e() {
    let (tx, rx) = tokio::sync::oneshot::channel();

    // Start server on port 6380 to avoid conflict with main server
    let server_handle = tokio::spawn(async move {
        let listener = TcpListener::bind("127.0.0.1:6380").await.unwrap();
        let db = Arc::new(db::DB::new());

        let mut rx = rx;
        loop {
            tokio::select! {
                res = listener.accept() => {
                    let (socket, _) = res.unwrap();
                    let db_clone = Arc::clone(&db);
                    tokio::spawn(async move {
                        connection::handle_connection(socket, db_clone).await;
                    });
                }
                _ = &mut rx => break,
            }
        }
    });

    // Give the server time to start
    tokio::time::sleep(Duration::from_millis(100)).await;
    eprintln!("Server should be started");

    // Connect client
    eprintln!("Connecting to server");
    let mut stream = TcpStream::connect("127.0.0.1:6380").await.unwrap();
    eprintln!("Connected");

    // Test PING
    eprintln!("Sending PING");
    stream.write_all(b"*1\r\n$4\r\nPING\r\n").await.unwrap();
    let mut buf = [0; 1024];
    eprintln!("Reading PING response");
    let n = stream.read(&mut buf).await.unwrap();
    eprintln!("Received: {:?}", &buf[..n]);
    assert_eq!(&buf[..n], b"+PONG\r\n");

    // Test SET
    eprintln!("Sending SET");
    stream.write_all(b"*3\r\n$3\r\nSET\r\n$3\r\nkey\r\n$5\r\nvalue\r\n").await.unwrap();
    eprintln!("Reading SET response");
    let n = stream.read(&mut buf).await.unwrap();
    eprintln!("Received: {:?}", &buf[..n]);
    assert_eq!(&buf[..n], b"+OK\r\n");

    // Test GET
    eprintln!("Sending GET");
    stream.write_all(b"*2\r\n$3\r\nGET\r\n$3\r\nkey\r\n").await.unwrap();
    eprintln!("Reading GET response");
    let n = stream.read(&mut buf).await.unwrap();
    eprintln!("Received: {:?}", &buf[..n]);
    assert_eq!(&buf[..n], b"$5\r\nvalue\r\n");

    // Test GET for non-existent key
    eprintln!("Sending GET for nonkey");
    stream.write_all(b"*2\r\n$3\r\nGET\r\n$6\r\nnonkey\r\n").await.unwrap();
    eprintln!("Reading GET nonkey response");
    let n = stream.read(&mut buf).await.unwrap();
    eprintln!("Received: {:?}", &buf[..n]);
    assert_eq!(&buf[..n], b"$-1\r\n");

    // Test SET with UTF-8 string for embedding
    eprintln!("Sending SET for embedding");
    stream.write_all(b"*3\r\n$3\r\nSET\r\n$7\r\nembkey1\r\n$11\r\nhello world\r\n").await.unwrap();
    let n = stream.read(&mut buf).await.unwrap();
    assert_eq!(&buf[..n], b"+OK\r\n");

    // Wait for embedding generation
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Test SIMILARITY
    eprintln!("Sending SIMILARITY");
    stream.write_all(b"*3\r\n$10\r\nSIMILARITY\r\n$7\r\nembkey1\r\n$1\r\n1\r\n").await.unwrap();
    let n = stream.read(&mut buf).await.unwrap();
    eprintln!("Received: {:?}", &buf[..n]);
    // Should be *1\r\n$7\r\nembkey1\r\n
    assert!(&buf[..n].starts_with(b"*1\r\n$7\r\nembkey1\r\n"));

    // Shutdown the server
    tx.send(()).unwrap();
    server_handle.await.unwrap();
}