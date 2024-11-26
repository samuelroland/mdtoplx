pub mod core;
pub mod models;
pub mod utils;
use crate::models::exo::*;

use colored::Colorize;
use core::core::try_parse_all_exos;
use models::md::MDFile;
use models::stats::CompilationStats;
use std::fs::exists;
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread::JoinHandle;
use utils::utils::{clone_repository, is_hidden};
use walkdir::WalkDir;

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
        clone_repository(REPOSITORY);
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

    stats.print();
}
