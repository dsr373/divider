mod core;
use crate::core::User;
mod backend;
use crate::backend::json_store;

fn main() {
    let user = User::new("John Doe");
    println!("Hello, {}!", user);
}
