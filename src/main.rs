mod core;
use crate::core::User;

fn main() {
    let user = User::new("John Doe");
    println!("Hello, {}!", user);
}
