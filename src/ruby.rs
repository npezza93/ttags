use inflector::string::singularize::to_singular;
use npezza93_tree_sitter_tags::{Tag as TSTag, TagsConfiguration, TagsContext};
use std::str;
use std::sync::LazyLock;
use streaming_iterator::StreamingIterator;
use tree_sitter::{Parser, Query, QueryCursor};

use crate::tag::Tag;

const DELEGATE_SCHEMA: &str = include_str!("../ruby/delegate.scm");
static DELEGATE_QUERY: LazyLock<Query> =
    LazyLock::new(|| Query::new(&tree_sitter_ruby::LANGUAGE.into(), DELEGATE_SCHEMA).unwrap());

pub fn config() -> TagsConfiguration {
    TagsConfiguration::new(
        tree_sitter_ruby::LANGUAGE.into(),
        include_str!("../ruby/tags.scm"),
        tree_sitter_ruby::LOCALS_QUERY,
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
        let original_name = str::from_utf8(tag_name).unwrap_or("");
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
    parser
        .set_language(&tree_sitter_ruby::LANGUAGE.into())
        .unwrap();
    parser.reset();

    let tree = parser.parse(docs, None).unwrap();

    let mut matches = cursor.matches(&DELEGATE_QUERY, tree.root_node(), docs);

    let mut is_match = false;
    let mut matchy1 = None;
    if let Some(matchy) = matches.next() {
        is_match = true;
        matchy1 = Some(matchy.captures()[1])
    };
    let is_prefix_match = matches.next().is_some();

    if is_match {
        if is_prefix_match {
            let prefix = matchy1.unwrap().node.utf8_text(docs).unwrap().to_owned();

            prefix[1..prefix.len()].to_string() + "_" + parsed_name
        } else {
            parsed_name.to_string()
        }
    } else {
        parsed_name.to_string()
    }
}

#[cfg(test)]
mod performance_regressions {
    use super::delegate_name;

    #[test]
    fn cached_delegate_query_handles_changing_options() {
        let cases: &[(&str, &[u8], &str)] = &[
            (
                "name",
                b"delegate :name, to: :author, prefix: true",
                "author_name",
            ),
            ("email", b"delegate :email, to: :author", "email"),
            (
                "name",
                b"delegate :name, to: :reader, prefix: false",
                "name",
            ),
            (
                "name",
                b"delegate :name, to: :reader, prefix: true",
                "reader_name",
            ),
            ("name", b"delegate :name", "name"),
            ("name", b"", "name"),
            (
                "café",
                "delegate :café, to: :écrivain, prefix: true".as_bytes(),
                "écrivain_café",
            ),
        ];
        for _ in 0..3 {
            for (name, docs, expected) in cases {
                assert_eq!(delegate_name(name, docs), *expected);
            }
        }
    }

    #[test]
    fn cached_delegate_query_is_usable_from_independent_threads() {
        std::thread::scope(|scope| {
            for _ in 0..4 {
                scope.spawn(|| {
                    assert_eq!(
                        delegate_name("name", b"delegate :name, to: :author, prefix: true"),
                        "author_name"
                    );
                });
            }
        });
    }
}
