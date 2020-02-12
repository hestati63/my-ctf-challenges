mod poly;
mod gf256;
mod encode;

use std::fs::File;
use std::io::Read;
use encode::encode;

fn read_file(file: &mut File) -> String {
    let mut buf = String::new();
    file.read_to_string(&mut buf).expect("Cannot read flag");
    buf
}

fn main() {
    let mut flag = File::open("flag").expect("Cannot open flag");
    let flag = read_file(&mut flag);
    encode(&flag).iter()
                    .for_each(|x| print!("{:02x} ", x));
}
