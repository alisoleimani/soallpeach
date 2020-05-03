use actix_web::{web, App, HttpResponse, HttpServer};
use actix_web::body::Body;
use bytes::Bytes;
use std::sync::atomic::{AtomicIsize, Ordering};
use std::str;

static mut COUNT: AtomicIsize = AtomicIsize::new(0);

async fn add(body: Bytes) -> HttpResponse {
    unsafe {
        COUNT.fetch_add(str::from_utf8_unchecked(&body).parse::<isize>().unwrap(), Ordering::SeqCst);
    }
    HttpResponse::Ok().body(Body::Empty)
}

async fn count() -> HttpResponse {
    unsafe {
        HttpResponse::Ok().body(format!("{}",COUNT.load(Ordering::SeqCst)))
    }
}


#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/", web::post().to(add))
            .route("/count", web::get().to(count))
    })
    .bind("127.0.0.1:80")?
    .run()
    .await
}

