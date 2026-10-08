//! Markdown rendering: ids, links, front matter, highlighting, alerts, escaping.

use pvas_web_docs::highlight::STYLESHEET;
use pvas_web_docs::markdown::Markdown;

const PAGE: &str = r#"+++
title = "Getting Started"
order = 1
+++

# Getting Started

See the [reference](/reference/) or [Epic](https://www.unrealengine.com/).

## Set up your project

```cpp
UCLASS()
class UThing : public UObject {};
```

> [!WARNING]
> Beta engine features.

## Set up your project

<script>alert(1)</script>
"#;

#[test]
fn heading_ids_match_the_html_and_repeat_safely() {
    let rendered = Markdown::new().render(PAGE, "/gantry/");
    let ids: Vec<&str> = rendered.headings.iter().map(|h| h.id.as_str()).collect();
    assert_eq!(
        ids,
        [
            "getting-started",
            "set-up-your-project",
            "set-up-your-project-1"
        ]
    );
    for id in ids {
        assert!(
            rendered.html.contains(&format!("id=\"{id}\"")),
            "{id} missing from the HTML"
        );
    }
}

#[test]
fn site_absolute_links_get_the_base_and_others_do_not() {
    let html = Markdown::new().render(PAGE, "/gantry/").html;
    assert!(html.contains("href=\"/gantry/reference/\""));
    assert!(html.contains("href=\"https://www.unrealengine.com/\""));
}

#[test]
fn front_matter_is_returned_and_not_rendered() {
    let rendered = Markdown::new().render(PAGE, "/gantry/");
    assert!(
        rendered
            .front_matter
            .contains("title = \"Getting Started\"")
    );
    assert!(!rendered.front_matter.contains("+++"));
    assert!(!rendered.html.contains("order = 1"));
}

#[test]
fn code_is_highlighted_with_prefixed_classes() {
    let html = Markdown::new().render(PAGE, "/gantry/").html;
    assert!(
        html.contains("class=\"hl-"),
        "no highlighting classes in: {html}"
    );
}

#[test]
fn alerts_render_and_raw_html_is_escaped() {
    let html = Markdown::new().render(PAGE, "/gantry/").html;
    assert!(html.contains("markdown-alert"), "alert missing: {html}");
    assert!(!html.contains("<script>"), "raw HTML passed through");
}

/// Code is colored from the theme's tokens, never fixed colors, so a product's hues restyle it.
#[test]
fn stylesheet_colors_from_the_theme() {
    for scope in [".hl-keyword", ".hl-string", ".hl-comment", ".hl-entity"] {
        assert!(STYLESHEET.contains(scope), "no rule for {scope}");
    }
    assert!(STYLESHEET.contains("var(--yeti-color-primary-text)"));
    assert!(
        !STYLESHEET.contains('#'),
        "a fixed color in the highlighting stylesheet"
    );
}
