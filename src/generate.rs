// Default logic for generating tags

use npezza93_tree_sitter_tags::{Tag as TSTag, TagsConfiguration, TagsContext};
use std::str;

use crate::tag::Tag;

pub fn default_generate_tags<'a>(
    create_tag: fn(&str, &str, &TSTag, &str) -> Tag,
    context: &'a mut TagsContext,
    config: &'a TagsConfiguration,
    filename: &'a str,
    contents: &'a [u8],
) -> Vec<Tag> {
    let tags = context.generate_tags(config, contents, None).unwrap().0;

    tags.flat_map(|tag| {
        let tag = tag.unwrap();
        let node_name = config.syntax_type_name(tag.syntax_type_id);
        let tag_name = &contents[tag.name_range.start..tag.name_range.end];
        let original_name = str::from_utf8(<&[u8]>::clone(&tag_name)).unwrap_or("");

        vec![create_tag(original_name, node_name, &tag, filename)]
    })
    .collect::<Vec<Tag>>()
}

pub fn default_create_tag<'a>(
    name: &'a str,
    node_name: &'a str,
    tag: &'a TSTag,
    filename: &'a str,
) -> Tag {
    let row = tag.span.start.row;

    let kind = match node_name {
        "function" => "f",
        "class" | "interface" => "c",
        "module" => "m",
        _ => node_name,
    };

    Tag::new(name, filename, row + 1, kind)
}
