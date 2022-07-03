mod core;
use crate::core::user::{self, User};

fn main() {
    let user = User::new("John Doe".to_string());
    println!("Hello, {}!", user);
}
