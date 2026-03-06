use corefetch::app::App;
use corefetch::cli::Invocation;

fn main() {
    let invocation = Invocation::from_env();
    let output = App::bootstrap(invocation).run();
    println!("{output}");
}
