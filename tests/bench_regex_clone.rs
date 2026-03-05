use std::time::Instant;

fn main() {
    let re = regex::Regex::new("test").unwrap();
    let start = Instant::now();
    for _ in 0..100000 {
        let _ = re.clone();
    }
    println!("Cloned 100000 times in {:?}", start.elapsed());
}
