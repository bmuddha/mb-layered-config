use magicblock_config::MagicBlockParams;

fn main() {
    let params = MagicBlockParams::try_new().unwrap();
    println!("params: {}", toml::to_string_pretty(&params).unwrap());
    println!("keypair: {}", params.validator.keypair.to_string());
}
