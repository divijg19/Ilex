use corefetch::app::App;
use corefetch::cli::Invocation;

fn main() {
    let invocation = Invocation::from_env();
    let app = App::bootstrap(invocation).unwrap_or_else(|error| {
        eprintln!("corefetch: {error}");
        std::process::exit(1);
    });
    let output = app.run();
    println!("{output}");
}
