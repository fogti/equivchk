pub mod boolcx;

fn main() {
    use std::process::exit;
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("invalid invocation; USAGE: equivchk FILENAME");
        exit(1);
    }

    let src = std::fs::read(&args[1]).expect("unable to read file");
    let src = std::str::from_utf8(&*src).expect("file isn't valid UTF-8");

    let document = boolcx::Document::parse(src);
    document.iter_permute();
}
