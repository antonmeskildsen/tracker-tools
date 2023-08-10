use std::process::{Command, Stdio};

pub fn main() {
    let mut child = Command::new("edf2asc")
        .arg("resources/Patterns_214_0.EDF")
        .stdout(Stdio::inherit())
        .spawn()
        .unwrap();
    child.wait().unwrap();
}
