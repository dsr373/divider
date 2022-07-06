use std::fmt;

#[derive(Eq, PartialEq, Hash)]
pub struct User {
    name: String,
}

impl User {
    pub fn new(name_: String) -> User {
        User { name: name_ }
    }
}

impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl fmt::Debug for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "User {}", self.name)
    }
}
