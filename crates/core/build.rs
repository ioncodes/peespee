use std::fmt::Write;
use std::{env, fs, path::PathBuf};

use heck::ToShoutySnakeCase;

const DATABASE: &str = "../../data/nids.txt";

fn main() {
    println!("cargo::rerun-if-changed={DATABASE}");

    let database = fs::read_to_string(DATABASE).expect("failed to read NID database");

    let mut constants = String::new();
    let mut names = String::new();
    for (index, line) in database.lines().enumerate() {
        let (nid, name) = line
            .split_once(' ')
            .unwrap_or_else(|| panic!("invalid NID NAME entry on line {}", index + 1));

        let prefix = &name[..name.len() - name.trim_start_matches('_').len()];
        let identifier = format!("{prefix}{}", name.to_shouty_snake_case());

        writeln!(constants, "pub(super) const {identifier}: u32 = {nid};").unwrap();
        writeln!(names, "        {identifier} => Some(\"{name}\"),").unwrap();
    }

    let output = format!(
        "{constants}
pub(super) fn name(nid: u32) -> Option<&'static str> {{
    match nid {{
{names}        _ => None,
    }}
}}
"
    );

    let output_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    fs::write(output_dir.join("nids.rs"), output).expect("failed to write generated NIDs");
}
