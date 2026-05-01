use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};

/// Converts Markdown text to Pango markup.
/// Handles basic markdown elements: bold, italic, headers, lists, code, code blocks.
/// All text content is escaped to prevent Pango markup injection.
pub fn markdown_to_pango(input: &str) -> String {
    let mut output = String::new();
    let parser = Parser::new_ext(input, Options::all());

    for event in parser {
        match event {
            Event::Start(tag) => match tag {
                Tag::Strong => {
                    output.push_str("<b>");
                }
                Tag::Emphasis => {
                    output.push_str("<i>");
                }
                Tag::CodeBlock(_) => {
                    output.push_str("<tt>");
                }
                Tag::Heading { level, .. } => {
                    use pulldown_cmark::HeadingLevel;
                    output.push_str(match level {
                        HeadingLevel::H1 => "<span size=\"xx-large\"><b>",
                        HeadingLevel::H2 => "<span size=\"x-large\"><b>",
                        HeadingLevel::H3 => "<span size=\"large\"><b>",
                        _ => "<span size=\"large\"><b>",
                    });
                }
                Tag::List(_) => {}
                Tag::Item => {
                    output.push_str("• ");
                }
                _ => {}
            },
            Event::End(tag_end) => match tag_end {
                TagEnd::Strong => {
                    output.push_str("</b>");
                }
                TagEnd::Emphasis => {
                    output.push_str("</i>");
                }
                TagEnd::CodeBlock => {
                    output.push_str("</tt>");
                }
                TagEnd::Heading(_) => {
                    output.push_str("</b></span>");
                }
                TagEnd::Item => {
                    output.push('\n');
                }
                _ => {}
            },
            Event::Text(text) => {
                let escaped = escape_pango(&text);
                output.push_str(&escaped);
            }
            Event::Code(code) => {
                // Inline code is already handled by tags, but standalone Event::Code can occur
                output.push_str("<tt>");
                output.push_str(&escape_pango(&code));
                output.push_str("</tt>");
            }
            Event::Html(html) => {
                // HTML in markdown - escape it to prevent injection
                output.push_str(&escape_pango(&html));
            }
            Event::SoftBreak | Event::HardBreak => {
                output.push('\n');
            }
            Event::Rule => {
                output.push_str("\n---\n");
            }
            _ => {}
        }
    }

    output
}

/// Escapes special Pango characters (&, <, >) to prevent markup injection.
/// This is a security mitigation (T-03-01) that prevents AI responses from
/// injecting arbitrary Pango markup.
pub fn escape_pango(text: &str) -> String {
    text.chars()
        .map(|c| match c {
            '&' => "&amp;".to_string(),
            '<' => "&lt;".to_string(),
            '>' => "&gt;".to_string(),
            c => c.to_string(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_pango_basic() {
        assert_eq!(escape_pango("hello"), "hello");
        assert_eq!(escape_pango("&"), "&amp;");
        assert_eq!(escape_pango("<"), "&lt;");
        assert_eq!(escape_pango(">"), "&gt;");
        assert_eq!(escape_pango("<script>"), "&lt;script&gt;");
        assert_eq!(escape_pango("&amp;"), "&amp;amp;");
    }

    #[test]
    fn test_markdown_to_pango_bold() {
        let input = "**bold text**";
        let output = markdown_to_pango(input);
        assert!(output.contains("<b>"));
        assert!(output.contains("bold text"));
        assert!(output.contains("</b>"));
    }

    #[test]
    fn test_markdown_to_pango_italic() {
        let input = "*italic text*";
        let output = markdown_to_pango(input);
        assert!(output.contains("<i>"));
        assert!(output.contains("italic text"));
        assert!(output.contains("</i>"));
    }

    #[test]
    fn test_markdown_to_pango_bold_italic() {
        let input = "***bold and italic***";
        let output = markdown_to_pango(input);
        assert!(output.contains("<b>") || output.contains("<i>"));
        assert!(output.contains("bold and italic"));
    }

    #[test]
    fn test_markdown_to_pango_header1() {
        let input = "# Header 1";
        let output = markdown_to_pango(input);
        assert!(output.contains("xx-large"));
        assert!(output.contains("<b>"));
        assert!(output.contains("Header 1"));
    }

    #[test]
    fn test_markdown_to_pango_header2() {
        let input = "## Header 2";
        let output = markdown_to_pango(input);
        assert!(output.contains("x-large"));
        assert!(output.contains("Header 2"));
    }

    #[test]
    fn test_markdown_to_pango_header3() {
        let input = "### Header 3";
        let output = markdown_to_pango(input);
        assert!(output.contains("large"));
        assert!(output.contains("Header 3"));
    }

    #[test]
    fn test_markdown_to_pango_inline_code() {
        let input = "`code snippet`";
        let output = markdown_to_pango(input);
        assert!(output.contains("<tt>"));
        assert!(output.contains("code snippet"));
        assert!(output.contains("</tt>"));
    }

    #[test]
    fn test_markdown_to_pango_code_block() {
        let input = "```rust\nfn main() {}\n```";
        let output = markdown_to_pango(input);
        assert!(output.contains("<tt>"));
        assert!(output.contains("fn main()"));
    }

    #[test]
    fn test_markdown_to_pango_list() {
        let input = "- Item 1\n- Item 2";
        let output = markdown_to_pango(input);
        assert!(output.contains("•"));
        assert!(output.contains("Item 1"));
        assert!(output.contains("Item 2"));
    }

    #[test]
    fn test_markdown_to_pango_escapes_html() {
        let input = "<script>alert('xss')</script>";
        let output = markdown_to_pango(input);
        assert!(output.contains("&lt;"));
        assert!(output.contains("&gt;"));
        assert!(!output.contains("<script>"));
    }

    #[test]
    fn test_markdown_to_pango_preserves_ampersand() {
        let input = "Fish & Chips";
        let output = markdown_to_pango(input);
        assert!(output.contains("&amp;"));
        assert!(!output.contains("Fish & Chips"));
    }

    #[test]
    fn test_markdown_to_pango_plain_text() {
        let input = "Just plain text";
        let output = markdown_to_pango(input);
        assert_eq!(output, "Just plain text");
    }

    #[test]
    fn test_markdown_to_pango_mixed_content() {
        let input = "This is **bold** and *italic* and `code`";
        let output = markdown_to_pango(input);
        assert!(output.contains("<b>"));
        assert!(output.contains("<i>"));
        assert!(output.contains("<tt>"));
        assert!(output.contains("bold"));
        assert!(output.contains("italic"));
        assert!(output.contains("code"));
    }
}
