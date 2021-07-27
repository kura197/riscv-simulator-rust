use std::fs;

fn main() {
    let filename = "test.txt";
    let contents = fs::read(filename).unwrap();

    for c in contents.iter() {
        println!("{}", c);
    }
}
