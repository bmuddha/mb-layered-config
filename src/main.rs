use std::env::args_os;

use magicblock_config::MagicBlockParams;

fn main() {
    let params = MagicBlockParams::try_new(args_os()).unwrap();
    println!("{params:?}")
}
