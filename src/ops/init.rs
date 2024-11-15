use std::fs;

pub fn init_rit() {
    match fs::create_dir_all(".rit/objects") {
        Ok(_) => {}
        Err(e) => println!("{e:?}"),
    }
}
