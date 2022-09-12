use std::fmt;
use serde::{Serialize, Deserialize};

pub type UserName = String;
pub type Amount = f32;


#[derive(Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct User {
    pub name: UserName
}

impl User {
    pub fn new(name: &str) -> User {
        User { name: name.to_owned() }
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

#[cfg(test)]
mod tests {
    use crate::core::user::User;

    #[test]
    fn can_print() {
        let user = User::new("Pinocchio");
        print!("{}", user);
        assert_eq!(user.to_string(), "Pinocchio");
    }

    #[test]
    fn can_debug() {
        let user = User::new("Pinocchio");
        print!("{:?}", user);
    }
}
