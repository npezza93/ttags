use inflector::string::singularize::to_singular;
use npezza93_tree_sitter_tags::{Tag as TSTag, TagsConfiguration, TagsContext};
use std::str;
use tree_sitter::{Parser, Query, QueryCursor};

use crate::tag::Tag;

const DELEGATE_SCHEMA: &str = include_str!("../ruby/delegate.scm");

pub fn config() -> TagsConfiguration {
    TagsConfiguration::new(
        npezza93_tree_sitter_ruby::language(),
        include_str!("../ruby/tags.scm"),
        npezza93_tree_sitter_ruby::LOCALS_QUERY,
    )
    .unwrap()
}

pub fn generate_tags<'a>(
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
        let docs = tag.docs.clone().unwrap_or_default().as_bytes().to_owned();

        let name = name_override(node_name, original_name, tag_name);

        if node_name == "macro" {
            if let Some(metadata) = &tag.metadata {
                match metadata.as_str() {
                    "has_one" | "belongs_to" => {
                        vec![
                            create_tag(&name, metadata.as_str(), &tag, filename),
                            create_tag(&format!("{}=", name), metadata.as_str(), &tag, filename),
                            create_tag(
                                &format!("build_{}", to_singular(&name)),
                                metadata.as_str(),
                                &tag,
                                filename,
                            ),
                            create_tag(
                                &format!("create_{}", to_singular(&name)),
                                metadata.as_str(),
                                &tag,
                                filename,
                            ),
                            create_tag(
                                &format!("create_{}!", to_singular(&name)),
                                metadata.as_str(),
                                &tag,
                                filename,
                            ),
                        ]
                    }
                    "has_many" => {
                        vec![
                            create_tag(&name, metadata.as_str(), &tag, filename),
                            create_tag(&format!("{}=", name), metadata.as_str(), &tag, filename),
                            create_tag(
                                &format!("{}_ids", to_singular(&name)),
                                metadata.as_str(),
                                &tag,
                                filename,
                            ),
                            create_tag(
                                &format!("{}_ids=", to_singular(&name)),
                                metadata.as_str(),
                                &tag,
                                filename,
                            ),
                        ]
                    }
                    "attr_accessor" => {
                        vec![
                            create_tag(&name, metadata.as_str(), &tag, filename),
                            create_tag(&format!("{}=", name), metadata.as_str(), &tag, filename),
                        ]
                    }
                    "attr_writer" => vec![create_tag(
                        &format!("{}=", name),
                        metadata.as_str(),
                        &tag,
                        filename,
                    )],
                    "delegate" => vec![create_tag(
                        &delegate_name(&name, &docs),
                        metadata.as_str(),
                        &tag,
                        filename,
                    )],
                    _ => vec![create_tag(&name, metadata.as_str(), &tag, filename)],
                }
            } else {
                vec![]
            }
        } else {
            vec![create_tag(&name, node_name, &tag, filename)]
        }
    })
    .collect::<Vec<Tag>>()
}

fn create_tag<'a>(name: &'a str, node_name: &'a str, tag: &'a TSTag, filename: &'a str) -> Tag {
    let row = tag.span.start.row;

    let kind = match node_name {
        "method" | "constructor" | "attr_reader" | "attr_writer" | "attr_accessor" | "delegate" => {
            "f"
        }
        "class" => "c",
        "module" => "m",
        "constant" => "C",
        "scope" | "has_many" | "has_one" | "belongs_to" | "singleton_method" => "F",
        _ => node_name,
    };

    Tag::new(name, filename, row + 1, kind)
}

fn name_override<'a>(node_name: &'a str, original_name: &'a str, tag_name: &'a [u8]) -> String {
    if original_name.starts_with(':') {
        original_name[1..tag_name.len()].to_string()
    } else if node_name == "constructor" {
        "new".to_string()
    } else {
        original_name.to_string()
    }
}

fn delegate_name<'a>(parsed_name: &'a str, docs: &'a [u8]) -> String {
    let mut parser = Parser::new();
    let mut cursor = QueryCursor::new();
    parser.set_language(&npezza93_tree_sitter_ruby::language()).unwrap();
    parser.reset();

    let tree = parser.parse(docs, None).unwrap();
    let query = Query::new(&npezza93_tree_sitter_ruby::language(), DELEGATE_SCHEMA).unwrap();

    let mut matches = cursor.matches(&query, tree.root_node(), docs);

    let name = if let Some(matchy) = matches.next() {
        if let Some(_prefix_match) = matches.next() {
            let prefix = matchy.captures[1].node.utf8_text(docs).unwrap().to_owned();

            prefix[1..prefix.len()].to_string() + "_" + parsed_name
        } else {
            parsed_name.to_string()
        }
    } else {
        parsed_name.to_string()
    };

    name
}
