//! The shared components' markup: what each variant renders.

use dioxus::prelude::*;

use pvas_web_components::{Card, Intro, NavLink};

/// Renders an element to HTML.
fn html(element: &Element) -> String {
    dioxus_ssr::render_element(element.clone())
}

#[test]
fn a_card_with_a_destination_is_a_link() {
    let out = html(&rsx! {
        Card { title: "Concepts", text: "How it fits.", href: "/gantry/concepts/".to_owned() }
    });
    assert!(out.starts_with("<a class=\"card pa-card\" href=\"/gantry/concepts/\""));
    assert!(out.contains("<h3>Concepts</h3>"));
}

#[test]
fn a_card_without_one_is_an_article_with_its_badge() {
    let out = html(&rsx! {
        Card { title: "TerraVoxel", text: "Planets.", badge: "coming soon".to_owned() }
    });
    assert!(out.starts_with("<article class=\"card\""));
    assert!(out.contains("<span class=\"badge\" data-variant=\"neutral\">coming soon</span>"));
    assert!(!out.contains("pa-card"));
}

#[test]
fn the_intro_leads_with_one_high_emphasis_action() {
    let out = html(&rsx! {
        Intro {
            eyebrow: "gantry",
            headline: "Content that knows its order.",
            lede: "A lede.",
            actions: vec![NavLink::new("Start", "/a/"), NavLink::new("Browse", "/b/")],
        }
    });
    assert!(out.contains("<p class=\"pa-eyebrow\">gantry</p>"));
    assert!(out.contains("<h1 id=\"headline\" class=\"pa-cursor\">"));
    assert_eq!(out.matches("data-emphasis=\"high\"").count(), 1);
    assert_eq!(out.matches("data-emphasis=\"medium\"").count(), 1);
}

#[test]
fn an_intro_without_actions_has_no_empty_cluster() {
    let out = html(&rsx! {
        Intro { eyebrow: "x", headline: "y", lede: "z", actions: vec![] }
    });
    assert!(!out.contains("cluster"));
}
