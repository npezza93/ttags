use tree_sitter_tags::{Tag, TagsContext, TagsConfiguration};
use std::str;

pub fn generate_tags(context: &mut TagsContext, config: &TagsConfiguration, filename: &str, contents: &[u8]) -> Vec<String> {
    let tags = context.generate_tags(config, contents, None).unwrap().0;

    tags.flat_map(|tag| {
        let tag = tag.unwrap();
        let node_name = config.syntax_type_name(tag.syntax_type_id);
        let tag_name = &contents[tag.name_range.start..tag.name_range.end];
        let name = str::from_utf8(<&[u8]>::clone(&tag_name)).unwrap_or("");

        match node_name {
            _ => vec![create_tag(&name, node_name, &tag, filename)]
        }
    }).collect::<Vec<String>>()
}

fn create_tag<'a>(name: &'a str, node_name: &'a str, tag: &'a Tag, filename: &'a str) -> String {
    let row = tag.span.start.row;

    let kind = match node_name {
        "method" | "function"  => "f",
        "class" => "c",
        _ => node_name
    };

    format!("{}\t{}\t:{}\t{}\n", name, filename, row + 1, kind)
}
