use crate::config::load_mwutil_config;

mod config;

fn main() {
    let config = load_mwutil_config()
        .unwrap();
    println!("{:?}", config);
}
