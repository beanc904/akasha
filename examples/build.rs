const GIT_HASH: &'static str = env!("GIT_HASH");

fn main() {
    println!("Git hash: {}", GIT_HASH);
}
