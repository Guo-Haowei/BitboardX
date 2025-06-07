mod engine;

fn main() {
    let a = 6;
    let b = 7;
    let c = engine::my_add(a, b);
    println!("This is a dummy test");
    println!("{} + {} = {}", a, b, c);
}
