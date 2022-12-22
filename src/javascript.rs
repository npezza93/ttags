use tree_sitter_tags::{Tag as TSTag, TagsContext, TagsConfiguration};
use std::str;

use crate::tag::Tag;

pub fn config() -> TagsConfiguration {
    TagsConfiguration::new(
        tree_sitter_javascript::language(),
        include_str!("../javascript/tags.scm"),
        tree_sitter_javascript::LOCALS_QUERY,
    ).unwrap()
}

pub fn generate_tags<'a>(context: &'a mut TagsContext, config: &'a TagsConfiguration, filename: &'a str, contents: &'a [u8]) -> Vec<Tag> {
    let tags = context.generate_tags(config, contents, None).unwrap().0;

    tags.flat_map(|tag| {
        let tag = tag.unwrap();
        let node_name = config.syntax_type_name(tag.syntax_type_id);
        let tag_name = &contents[tag.name_range.start..tag.name_range.end];
        let original_name = str::from_utf8(<&[u8]>::clone(&tag_name)).unwrap_or("");

        vec![create_tag(original_name, node_name, &tag, filename)]
    }).collect::<Vec<Tag>>()
}

fn create_tag<'a>(name: &'a str, node_name: &'a str, tag: &'a TSTag, filename: &'a str) -> Tag {
    let row = tag.span.start.row;

    let kind = match node_name {
        "method" | "function"  => "f",
        "class" => "c",
        _ => node_name
    };

    Tag::new(name, filename, row + 1, kind)
}
