use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        println!("xtask stub: handling {:?}", &args[1..]);
    } else {
        println!("xtask stub executing successfully");
    }
}
