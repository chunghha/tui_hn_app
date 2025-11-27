use crate::internal::models::ArticleElement;
use scraper::{Html, Selector};

pub fn parse_article_html(html: &str) -> Vec<ArticleElement> {
    let document = Html::parse_document(html);
    let mut elements = Vec::new();

    // Select the main content. This is a heuristic.
    // In a real app, we might use something like readability.js logic.
    // For now, we'll try to find common article containers or just parse the body.
    let _body_selector = Selector::parse("body").unwrap();

    // Simple approach: Iterate over all direct children of body or a main container
    // and convert them to ArticleElements.
    // Since we don't know the structure, let's just traverse important tags.

    // Better approach for generic HTML:
    // Select all p, h1-h6, pre, ul/ol, table, img tags in document order
    // and convert them.

    let selector =
        Selector::parse("p, h1, h2, h3, h4, h5, h6, pre, ul, ol, table, img, blockquote").unwrap();

    for element in document.select(&selector) {
        let tag_name = element.value().name();

        match tag_name {
            "p" => {
                let text = element.text().collect::<Vec<_>>().join("");
                if !text.trim().is_empty() {
                    elements.push(ArticleElement::Paragraph(text.trim().to_string()));
                }
            }
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                let level = tag_name[1..].parse::<usize>().unwrap_or(1);
                let text = element.text().collect::<Vec<_>>().join("");
                elements.push(ArticleElement::Heading(level, text.trim().to_string()));
            }
            "pre" => {
                // Check for code inside pre
                let code_selector = Selector::parse("code").unwrap();
                let code_text = if let Some(code_elem) = element.select(&code_selector).next() {
                    code_elem.text().collect::<Vec<_>>().join("")
                } else {
                    element.text().collect::<Vec<_>>().join("")
                };

                // Try to find language class
                let lang = element
                    .value()
                    .attr("class")
                    .or_else(|| {
                        element
                            .select(&code_selector)
                            .next()
                            .and_then(|c| c.value().attr("class"))
                    })
                    .map(|c| c.to_string());

                elements.push(ArticleElement::CodeBlock {
                    lang,
                    code: code_text,
                });
            }
            "ul" | "ol" => {
                let li_selector = Selector::parse("li").unwrap();
                let items: Vec<String> = element
                    .select(&li_selector)
                    .map(|li| li.text().collect::<Vec<_>>().join("").trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();

                if !items.is_empty() {
                    elements.push(ArticleElement::List(items));
                }
            }
            "table" => {
                let tr_selector = Selector::parse("tr").unwrap();
                let mut rows = Vec::new();

                for tr in element.select(&tr_selector) {
                    let td_selector = Selector::parse("td, th").unwrap();
                    let cols: Vec<String> = tr
                        .select(&td_selector)
                        .map(|td| td.text().collect::<Vec<_>>().join("").trim().to_string())
                        .collect();
                    if !cols.is_empty() {
                        rows.push(cols);
                    }
                }

                if !rows.is_empty() {
                    elements.push(ArticleElement::Table(rows));
                }
            }
            "img" => {
                if let Some(src) = element.value().attr("src") {
                    let alt = element.value().attr("alt").unwrap_or(src);
                    elements.push(ArticleElement::Image(alt.to_string()));
                }
            }
            "blockquote" => {
                let text = element.text().collect::<Vec<_>>().join("");
                if !text.trim().is_empty() {
                    elements.push(ArticleElement::Quote(text.trim().to_string()));
                }
            }
            _ => {}
        }
    }

    elements
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_html() {
        let html = r#"
            <html>
                <body>
                    <h1>Title</h1>
                    <p>Paragraph 1</p>
                    <p>Paragraph 2</p>
                </body>
            </html>
        "#;
        let elements = parse_article_html(html);
        assert_eq!(elements.len(), 3);
        assert!(matches!(elements[0], ArticleElement::Heading(1, _)));
        assert!(matches!(elements[1], ArticleElement::Paragraph(_)));
    }

    #[test]
    fn test_parse_code_block() {
        let html = r#"
            <pre><code class="rust">fn main() {}</code></pre>
        "#;
        let elements = parse_article_html(html);
        assert_eq!(elements.len(), 1);
        if let ArticleElement::CodeBlock { lang, code } = &elements[0] {
            assert_eq!(lang.as_deref(), Some("rust"));
            assert_eq!(code, "fn main() {}");
        } else {
            panic!("Expected CodeBlock");
        }
    }
}
