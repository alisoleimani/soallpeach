use tokio::net::TcpStream;
use kmp;
use std::{net::Shutdown, str};
use tokio::net::TcpListener;
use tokio::prelude::*;
use std::sync::atomic::{AtomicIsize, Ordering};

static mut COUNT: AtomicIsize = AtomicIsize::new(0);
const OK: &[u8] = b"HTTP/1.1 200 OK\r\n\r\n";
const CONTENT_LEN: &[u8] = b"Content-Length:";
const NEW_LINE: &[u8] = b"\r\n";
const HEADER_END: &[u8] = b"\r\n\r\n";
async fn handle_client(mut stream: TcpStream) {
    let mut header = [0 as u8; 2048];
    
    match stream.read(&mut header).await {
        Ok(size) => {
            if header[0] == ('G' as u8) {
                unsafe {
                    stream.write_all(OK).await;
                    stream.write_all(format!("{}", COUNT.load(Ordering::SeqCst)).as_bytes()).await;
                }
            } else {
                let cls = kmp::kmp_find(CONTENT_LEN, &header).unwrap();
                let cln = kmp::kmp_find(NEW_LINE, &header[cls+CONTENT_LEN.len()..]).unwrap();
                let len = size;
                unsafe {
                    let cl = str::from_utf8_unchecked(&header[cls+CONTENT_LEN.len()+1..cls+CONTENT_LEN.len()+cln]).parse::<usize>().unwrap();
                    let end = kmp::kmp_find(HEADER_END, &header[cls+CONTENT_LEN.len()+cln..]).unwrap();
                    let header_len = cls + CONTENT_LEN.len() + cln + end + HEADER_END.len();
                    let need_more = header_len  + cl - len;
                    if need_more > 0 {
                        stream.read_exact(&mut header[len..len + need_more]).await;
                    }
                    let input =  str::from_utf8_unchecked(&header[header_len..header_len+cl]).parse::<isize>().unwrap();
                    COUNT.fetch_add(input, Ordering::SeqCst);
                }
                stream.write_all(OK).await;
            }
        },
        Err(_) => {}
    }
    stream.shutdown(Shutdown::Both).unwrap();
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut listener = TcpListener::bind("0.0.0.0:80").await?;

    loop {
        let (socket, _) = listener.accept().await?;
        handle_client(socket).await;
    }
}


