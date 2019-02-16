use pathutils::resolve;

fn main() {
    println!("{}", resolve("/test", "../me.txt").unwrap());
}
