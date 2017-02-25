use std::io::prelude::*;

#[derive(Debug, Clone)]
pub struct User {
    pub name: String,
    pub pass: String,
    pub role: String,
    pub path: String,
    pub cur_dir: String,
}

impl User {
    pub fn new() -> User {
        User {
            name: String::new(),
            pass: "".to_string(),
            role: "user".to_string(),
            path: "".to_string(),
            cur_dir: "".to_string(),
        }
    }
}
