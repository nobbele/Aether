# Aether

Minimal logging library for Rust that uses explicit and configurable endpoints.

It supports automatic archiving of old files if using the `archive` feature, otherwise it adds a `.bk` extension.

## Example
```rs
use aether::log;

#[derive(Debug, Hash)]
enum Endpoint {
    Foo,
    Bar,
    Baz,
}

fn main() {
    let _keep = aether::init()
        .format(|log| {
            format!(
                "{} [{:?}] {}",
                log.time.format("%T.%3f"),
                log.endpoint,
                log.text
            )
        })
        .base_path("logs")
        .setup(Endpoint::Foo, |ep| ep.no_path())
        .setup(Endpoint::Bar, |ep| ep.path("output.log"))
        .setup(Endpoint::Baz, |ep| ep.path("baz.log").silent())
        .build();

    log!(Endpoint::Foo, "Hello World! {}", 0);
    log!(Endpoint::Baz, "I'm in the world!");
    log!(Endpoint::Bar, "Goodbye World! {}", 1);
}
```