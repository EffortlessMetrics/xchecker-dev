use regex::Regex;

fn main() {
    let re = Regex::new(r"AIzaSy[A-Za-z0-9_-]{33}").unwrap();
    let token = "AIzaSyABCDEFGHIJKLMNOPQRSTUVWXYZ0123456";

    println!("token1 matches: {}", re.is_match(token));
}
