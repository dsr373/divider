use std::fmt;
// use std::collections::HashSet;
// use std::sync::RwLock;

// use rand::Rng;
// use rand::prelude::ThreadRng;

#[derive(Eq, PartialEq, Clone, Hash)]
pub struct User {
    // pub id: usize,
    pub name: String
}

// static mut allUserIds: RwLock<HashSet<usize>> = RwLock::new(HashSet::new());

impl User {

    // fn new_id() -> usize {
    //     let mut rng = rand::thread_rng();
    //     let mut id: usize = rng.gen::<usize>();
    //     while {
    //         let all_ids = allUserIds.read().unwrap();
    //         all_ids.contains(&id)
    //     } {
    //         id = rng.gen::<usize>();
    //     }
    //     allUserIds.write().unwrap().insert(id);
    //     return id;
    // }

    pub fn new(name: &str) -> User {
        User { name: name.to_string() }
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
    }

    #[test]
    fn can_debug() {
        let user = User::new("Pinocchio");
        print!("{:?}", user);
    }


}
