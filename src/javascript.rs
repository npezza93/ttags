use tree_sitter_tags::{Tag, TagsContext, TagsConfiguration};
use std::str;

pub fn config() -> TagsConfiguration {
    TagsConfiguration::new(
        tree_sitter_javascript::language(),
        include_str!("../javascript/tags.scm"),
        tree_sitter_javascript::LOCALS_QUERY,
    ).unwrap()
}

pub fn generate_tags(context: &mut TagsContext, config: &TagsConfiguration, filename: &str, contents: &[u8]) -> Vec<String> {
    let tags = context.generate_tags(config, contents, None).unwrap().0;

    tags.flat_map(|tag| {
        let tag = tag.unwrap();
        let node_name = config.syntax_type_name(tag.syntax_type_id);
        let tag_name = &contents[tag.name_range.start..tag.name_range.end];
        let original_name = str::from_utf8(<&[u8]>::clone(&tag_name)).unwrap_or("");
        let docs = tag.docs.clone().unwrap_or_else(|| "".to_string()).as_bytes().to_owned();

        let name = name_override(node_name, original_name, tag_name, &docs);

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

    format!("{}\t{}\t{};\"\t{}\n", name, filename, row + 1, kind)
}

fn name_override<'a>(_node_name: &'a str, original_name: &'a str, _tag_name: &'a [u8], _docs: &'a [u8]) -> String {
    original_name.to_string()
}
