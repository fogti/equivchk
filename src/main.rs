pub mod boolcx;

// https://mastodon.social/@pingveno/102227428277791031
fn is_all_same<T: Eq>(slice: &[T]) -> bool {
    slice
        .get(0)
        .map(|first| slice.iter().all(|x| x == first))
        .unwrap_or(true)
}

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

    let is_same = document.iter_permute();
    println!("the given terms are {}", if is_same {
        "equivalent"
    } else {
        "not equivalent"
    });
}
