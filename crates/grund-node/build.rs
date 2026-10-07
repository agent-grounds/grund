// §FS-distribution.3.2.3.3: supported addons are unwind-capable Node-API libraries.
fn main() {
    if std::env::var("CARGO_CFG_PANIC").as_deref() == Ok("abort") {
        panic!("grund-node requires panic=unwind");
    }
    println!(
        "cargo:rustc-env=GRUND_NODE_TARGET={}",
        std::env::var("TARGET").unwrap()
    );
    napi_build::setup();
}
