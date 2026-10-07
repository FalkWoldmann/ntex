//! Hello-world server on the compio runtime with configurable io_uring setup
//! flags, for load testing.
//!
//! `cargo run --release --example compio-flags --features compio`
//!
//! Environment:
//! - `ADDR` (default `127.0.0.1:8080`)
//! - `WORKERS` (default: available parallelism)
//! - `IOURING_FLAGS`: `none` (default), `coop` (`COOP_TASKRUN`) or `defer`
//!   (`SINGLE_ISSUER | DEFER_TASKRUN`)

use std::io;

use ntex::http::{HttpService, Request, Response};
use ntex::rt::compio::{ProactorBuilder, set_proactor_builder};

fn proactor(flags: &str) -> ProactorBuilder {
    let mut builder = ProactorBuilder::new();
    match flags {
        "none" => &mut builder,
        "coop" => builder.coop_taskrun(true),
        "defer" => builder.single_issuer(true).defer_taskrun(true),
        other => panic!("unknown IOURING_FLAGS {other:?}"),
    };
    builder
}

fn main() -> io::Result<()> {
    let flags = std::env::var("IOURING_FLAGS").unwrap_or_else(|_| "none".into());
    let addr = std::env::var("ADDR").unwrap_or_else(|_| "127.0.0.1:8080".into());
    let workers = std::env::var("WORKERS")
        .ok()
        .map(|w| w.parse().expect("WORKERS must be a number"))
        .unwrap_or_else(|| std::thread::available_parallelism().map_or(1, |n| n.get()));

    assert!(set_proactor_builder(proactor(&flags)), "set once");
    eprintln!("ntex on compio: {addr}, {workers} workers, IOURING_FLAGS={flags}");

    ntex::rt::System::build()
        .name("compio-flags")
        .build(ntex::rt::DefaultRuntime)
        .block_on(async move {
            ntex::server::build()
                .workers(workers)
                .bind("hello", addr, ntex::SharedCfg::default(), async |_| {
                    HttpService::new(async |_: Request| {
                        Ok::<_, io::Error>(Response::Ok().body("Hello from ntex!\n"))
                    })
                })?
                .run()
                .await
        })
}
