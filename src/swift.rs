use npezza93_tree_sitter_tags::{Tag as TSTag, TagsConfiguration};
use std::str;

use crate::default_generate_tags;
use crate::tag::Tag;

pub fn config() -> TagsConfiguration {
    TagsConfiguration::new(
        tree_sitter_swift::language(),
        include_str!("../swift/tags.scm"),
        "",
    )
    .unwrap()
}

default_generate_tags!();

fn create_tag<'a>(name: &'a str, node_name: &'a str, tag: &'a TSTag, filename: &'a str) -> Tag {
    let row = tag.span.start.row;

    let kind = match node_name {
        "method" => "f",
        "class" => "c",
        _ => node_name,
    };

    Tag::new(name, filename, row + 1, kind)
}
