use std::fmt;

pub struct User {
    name: String,
    balance: f32
}

impl User {
    pub fn new(name_: String) -> User {
        User { name: name_, balance: 0.0 }
    }
}

impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl fmt::Debug for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "User {}: {}", self.name, self.balance)
    }
}