use std::fs;

pub fn add_rit() {
    match fs::create_dir_all(".rit/objects") {
        Ok(_) => {}
        Err(e) => println!("{e:?}"),
    }

    println!("initialized!")
}
