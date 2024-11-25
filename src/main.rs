use comrak::nodes::NodeValue;
use comrak::{parse_document, Arena, Options};
use std::fs::read_to_string;


fn append_to_option(text: &mut Option<String>, to_append: String) {
    match text {
        Some(i) => {
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

fn main() {
    let content = read_to_string(EXAMPLE_FILE).unwrap();

    // The returned nodes are created in the supplied Arena, and are bound by its lifetime.
    let arena = Arena::new();

    // Parse the document into a root `AstNode`
    let root = parse_document(&arena, &content, &Options::default());

    // dbg!(root);
    // Building exo medata
    let mut title: Option<String> = None;
    let mut instruction: Option<String> = None;
    let mut should_print: Option<String> = None;
    let mut solution_content: Option<String> = None;

    let mut in_details = false;

    let mut is_title = true;

    for node in root.descendants() {
        if let NodeValue::Heading(ref text) = node.data.borrow().value {
            print!("Heading level {} with content ", text.level);
            is_title = true;
        }

        if let NodeValue::Text(ref text) = node.data.borrow().value {
            println!("text: {text}");
            if is_title {
                title = Some(text.clone());
                is_title = false;
            } else {
                append_to_option(&mut instruction, text.clone());
            }
        }
        if let NodeValue::HtmlBlock(ref text) = node.data.borrow().value {
            print!("HTML: {}", text.literal);
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
            println!("code block: {} with info {}", text.literal, text.info);

            if in_details {
                solution_content = Some(text.literal.clone());
            } else if text.info.is_empty() || text.info == "text" {
                // if it is a textual output
                append_to_option(&mut should_print, text.literal.clone());
            }
        }
    }
    println!("------");
    println!(
        "name = '{}'\ninstruction = \"\"\"\n{}\n\"\"\"",
        title.unwrap_or("??".to_string()).replace("'", "\\'"),
        instruction.unwrap_or("??".to_string()).replace("'", "\\'")
    );

    println!("Should print:\n{}", first_lines(should_print));
    println!("Solution content:\n{}", first_lines(solution_content));
}
