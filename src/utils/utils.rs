use std::process::Command;

use walkdir::DirEntry;

pub fn append_to_option(text: &mut Option<String>, to_append: String) {
    match text {
        Some(i) => {
            i.push_str("\n");
            i.push_str(&to_append);
        }
        None => *text = Some(to_append),
    }
}

pub fn first_lines(content: Option<String>) -> String {
    content
        .unwrap_or("??".to_string())
        .lines()
        .map(|l| {
            let mut copy = l.to_string();
            copy.push_str("\n");
            copy
        })
        .take(4)
        .collect::<String>()
}

pub fn clone_repository(repository: &str) {
    Command::new("/usr/bin/git")
        .arg("clone")
        .arg(repository)
        .output()
        .expect(&format!("failed to clone repository {}", repository));
}

// from docs https://docs.rs/walkdir/latest/walkdir/
pub fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}
