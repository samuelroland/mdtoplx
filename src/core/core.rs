use std::fs::read_to_string;

use crate::{models::md::MDFile, parse_exo};

pub fn try_parse_all_exos(mdfiles: &mut Vec<MDFile>) {
    for md in mdfiles {
        match read_to_string(&md.path) {
            Ok(content) => {
                // Exclude all files that don't have a main function
                if content.contains("int main") {
                    md.parsed_exo = Some(parse_exo(content));
                }
            }
            Err(err) => eprintln!(
                "Impossibe to read file {} with {}",
                md.path.clone().into_os_string().into_string().unwrap(),
                err
            ),
        }
    }
}
