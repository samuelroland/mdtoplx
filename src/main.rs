use colored::*;
use comrak::nodes::NodeValue;
use comrak::{parse_document, Arena, Options};
use std::ffi::OsString;
use std::fs::{exists, read_to_string};
use std::os::unix::thread::JoinHandleExt;
use std::path::{Path, PathBuf};
use std::process::{exit, Command, Stdio};
use std::thread::{self, JoinHandle};
use walkdir::{DirEntry, WalkDir};

#[derive(Default, Debug, Clone)]
struct Exo {
    title: Option<String>,            // from Heading level 1
    instruction: Option<String>,      // from all available text
    should_print: Option<String>,     // from code block not in cpp
    solution_content: Option<String>, //
}

#[derive(Debug)]
struct MDFile {
    path: PathBuf,
    chapter: OsString,
    parsed_exo: Option<Result<Exo, String>>, // exo can be parsed or not
}

// PRG2
// static REPOSITORY: &str = "https://github.com/PRG2-HEIGVD/PRG2_Recueil_Exercices";
// static REPOSITORY_NAME: &str = "PRG2_Recueil_Exercices";
// static DEBUG: bool = true;
// static BAT_SHOW: bool = false;
// static OUTFILE: &str = "main.c";
// static COMPILER: &str = "/usr/bin/gcc";
// static COMPILER_ARG: &str = "";
// static FIX_HEADERS: &str = "";

// PRG1
static REPOSITORY: &str = "https://github.com/PRG1-HEIGVD/PRG1_Recueil_Exercices";
static REPOSITORY_NAME: &str = "PRG1_Recueil_Exercices";
static DEBUG: bool = true;
static BAT_SHOW: bool = false;
static OUTFILE: &str = "main.cpp";
static COMPILER: &str = "/usr/bin/g++";
static COMPILER_ARG: &str = "-std=c++2a";
static FIX_HEADERS: &str = "#include <iostream>\n#include <vector>\n#include <array>\n#include <string>\n#include <cstddef>\n\nusing namespace std;";

fn append_to_option(text: &mut Option<String>, to_append: String) {
    match text {
        Some(i) => {
            i.push_str("\n");
            i.push_str(&to_append);
        }
        None => *text = Some(to_append),
    }
}

fn first_lines(content: Option<String>) -> String {
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

fn parse_exo(content: String) -> Result<Exo, String> {
    let arena = Arena::new();
    let root = parse_document(&arena, &content, &Options::default());

    // Building exo medata
    let mut in_details = false;
    let mut exo: Exo = Exo::default();

    let mut is_title = true;

    // Visit all nodes of the Markdown AST
    for node in root.descendants() {
        if let NodeValue::Heading(_) = node.data.borrow().value {
            // print!("Heading level {} with content ", text.level);
            is_title = true; // so we can take its content at the next Text node
        }

        if let NodeValue::Text(ref text) = node.data.borrow().value {
            // println!("text: {text}");
            if is_title {
                exo.title = Some(text.clone());
                is_title = false;
            } else {
                append_to_option(&mut exo.instruction, text.clone());
            }
        }
        if let NodeValue::HtmlBlock(ref text) = node.data.borrow().value {
            // print!("HTML: {}", text.literal);
            // dbg!(node);
            if text.literal.contains("</details>") {
                in_details = false;
                continue;
            }
            if !text.literal.contains("Solution") {
                continue;
            }
            in_details = true;
        }

        if let NodeValue::CodeBlock(ref text) = node.data.borrow().value {
            // println!("code block: {} with info {}", text.literal, text.info);

            if in_details && text.info == "cpp" {
                //If the file has a single line this is probably not a file to compile
                if text.literal.lines().collect::<Vec<&str>>().len() > 1 {
                    exo.solution_content = Some(text.literal.clone());
                }
            } else if text.info.is_empty() || text.info == "text" {
                // if it is a textual output
                append_to_option(&mut exo.should_print, text.literal.clone());
            }
        }
    }
    Ok(exo.clone())
}

fn try_parse_all_exos(mdfiles: &mut Vec<MDFile>) {
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

fn clone_repository() {
    Command::new("/usr/bin/git")
        .arg("clone")
        .arg(REPOSITORY)
        .output()
        .expect(&format!("failed to clone repository {}", REPOSITORY));
}

// from docs https://docs.rs/walkdir/latest/walkdir/
fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

#[derive(Default, Debug)]
struct CompilationStats {
    success: u16,
    failed: u16,
}

fn try_to_build_cpp(code: &String, stats: &mut CompilationStats, inject_headers: String) {
    let mut target = "target/".to_string();
    target.push_str(OUTFILE);

    let _ = std::fs::write(&target, inject_headers + "\n" + code);
    let codecpy = code.clone();
    match Command::new(COMPILER)
        .arg(COMPILER_ARG)
        .arg("-o")
        .arg("target/main")
        .arg(&target)
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .output()
    {
        Ok(output) => {
            let stdout = String::from_utf8(output.stdout).unwrap();
            let stderr = String::from_utf8(output.stderr).unwrap();
            if output.status.success() {
                println!("{}", "Build is working!".green());
                stats.success += 1;
            } else {
                println!(
                    "Build failed! \n{}on FILE:\n{}",
                    stderr.red(),
                    codecpy.cyan()
                );
                // println!("Build failed! \n{}", stderr);
                if BAT_SHOW {
                    let _ = Command::new("/usr/bin/bat")
                        .arg("--paging")
                        .arg("never")
                        .arg(&target.clone())
                        .stdin(Stdio::piped())
                        .spawn();
                }

                stats.failed += 1;
            }
        }
        Err(err) => eprintln!("Build failed: {}", err),
    }
}

fn main() {
    if !exists(REPOSITORY_NAME).unwrap_or_default() {
        clone_repository();
    }
    let walker = WalkDir::new(REPOSITORY_NAME);
    let mut mdfiles: Vec<MDFile> = vec![];
    for entry in walker.into_iter().filter_entry(|e| !is_hidden(e)) {
        let entry = entry.unwrap();

        let path = Path::new(entry.path());
        let path_str = path.file_name().unwrap().to_str().unwrap();
        if !path_str.ends_with(".md") {
            continue;
        }
        if path_str.ends_with("README.md") {
            continue;
        }
        // if !path_str.contains("09") {
        //     continue;
        // }

        // dbg!(&entry);
        if let Some(parent_name) = path.parent().unwrap().file_name() {
            mdfiles.push(MDFile {
                path: entry.path().to_path_buf(),
                chapter: parent_name.to_os_string(),
                parsed_exo: None,
            });
        }
    }
    println!("Parsing {} files...", mdfiles.len());
    try_parse_all_exos(&mut mdfiles);
    dbg!(&mdfiles);
    println!("Done parsing {} files...", mdfiles.len());

    println!("Try building a few files");

    let mut stats = CompilationStats::default();
    let mut threads: Vec<JoinHandle<()>> = Vec::default();
    threads.reserve(mdfiles.len());
    for md in mdfiles {
        if let Some(Ok(exo)) = md.parsed_exo {
            if let Some(content) = exo.solution_content {
                println!("Starting build of {}", md.path.to_str().unwrap());
                try_to_build_cpp(&content, &mut stats, FIX_HEADERS.to_string());
            }
        }
    }

    dbg!(stats);
}
