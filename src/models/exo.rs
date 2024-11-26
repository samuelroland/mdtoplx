use comrak::nodes::NodeValue;
use comrak::Arena;
use comrak::{parse_document, Options};

use crate::utils::utils::append_to_option;

#[derive(Default, Debug, Clone)]
pub struct Exo {
    pub title: Option<String>,            // from Heading level 1
    pub instruction: Option<String>,      // from all available text
    pub should_print: Option<String>,     // from code block not in cpp
    pub solution_content: Option<String>, //
}

pub fn parse_exo(content: String) -> Result<Exo, String> {
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
