use aether::slog;

#[derive(Debug, Hash)]
enum Endpoint {
    Foo,
    Bar,
    Baz,
}

fn main() {
    let _keep = aether::init()
        .base_path("logs")
        .setup(Endpoint::Foo, |ep| ep.no_path())
        .setup(Endpoint::Bar, |ep| ep.path("scoped-output.log"))
        .setup(Endpoint::Baz, |ep| ep.path("scoped-baz.log").silent())
        .build();

    aether::scoped(Endpoint::Foo, || {
        slog!("Hello World! {}", 0);
    });
    aether::scoped(Endpoint::Baz, || {
        slog!("I'm in the world!");
    });
    aether::scoped(Endpoint::Bar, || {
        slog!("Goodbye World! {}", 1);
    });
}
