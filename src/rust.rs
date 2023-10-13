use npezza93_tree_sitter_tags::{Tag as TSTag, TagsConfiguration, TagsContext};
use std::str;

use crate::generate;
use crate::tag::Tag;

pub fn config() -> TagsConfiguration {
    TagsConfiguration::new(
        tree_sitter_rust::language(),
        include_str!("../rust/tags.scm"),
        "",
    )
    .unwrap()
}

pub fn generate_tags<'a>(
    context: &'a mut TagsContext,
    config: &'a TagsConfiguration,
    filename: &'a str,
    contents: &'a [u8],
) -> Vec<Tag> {
    generate::default_generate_tags(create_tag, context, config, filename, contents)
}

fn create_tag<'a>(name: &'a str, node_name: &'a str, tag: &'a TSTag, filename: &'a str) -> Tag {
    let row = tag.span.start.row;

    let kind = match node_name {
        "method" | "function" => "f",
        "macro" => "F",
        "class" | "interface" => "c",
        "module" => "m",
        "constant" => "C",
        _ => node_name,
    };

    Tag::new(name, filename, row + 1, kind)
}
