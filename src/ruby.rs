use tree_sitter_tags::{Tag, TagsContext, TagsConfiguration};
use tree_sitter::{Parser, QueryCursor, Query};
use std::str;
use inflector::string::singularize::to_singular;

const DELEGATE_SCHEMA: &str = include_str!("../ruby/delegate.scm");

pub fn config() -> TagsConfiguration {
    TagsConfiguration::new(
        tree_sitter_ruby::language(),
        include_str!("../ruby/tags.scm"),
        tree_sitter_ruby::LOCALS_QUERY,
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
            "attr_accessor" => {
                vec![
                    create_tag(&name, node_name, &tag, filename),
                    create_tag(&format!("{}=", name), node_name, &tag, filename)
                ]
            },
            "has_many" => {
                vec![
                    create_tag(&name, node_name, &tag, filename),
                    create_tag(&format!("{}=", name), node_name, &tag, filename),
                    create_tag(&format!("{}_ids", to_singular(&name)), node_name, &tag, filename),
                    create_tag(&format!("{}_ids=", to_singular(&name)), node_name, &tag, filename)
                ]
            },
            "has_one" | "belongs_to" => {
                vec![
                    create_tag(&name, node_name, &tag, filename),
                    create_tag(&format!("{}=", name), node_name, &tag, filename),
                    create_tag(&format!("build_{}", to_singular(&name)), node_name, &tag, filename),
                    create_tag(&format!("create_{}", to_singular(&name)), node_name, &tag, filename),
                    create_tag(&format!("create_{}!", to_singular(&name)), node_name, &tag, filename)
                ]
            }
            _ => vec![create_tag(&name, node_name, &tag, filename)]
        }
    }).collect::<Vec<String>>()
}

fn create_tag<'a>(name: &'a str, node_name: &'a str, tag: &'a Tag, filename: &'a str) -> String {
    let row = tag.span.start.row;

    let kind = match node_name {
        "method" | "constructor" | "attr_reader" | "attr_writer" | "attr_accessor" => "f",
        "class" => "c",
        "module" => "m",
        "constant" => "C",
        "scope" | "has_many" | "has_one" | "belongs_to" | "singleton_method" => "F",
        _ => node_name
    };

    format!("{}\t{}\t{};\"\t{}\n", name, filename, row + 1, kind)
}

fn name_override<'a>(node_name: &'a str, original_name: &'a str, tag_name: &'a [u8], docs: &'a [u8]) -> String {
    let mut name =
        if original_name.starts_with(':') {
            original_name[1..tag_name.len()].to_string()
        } else {
            original_name.to_string()
        };

    name = match node_name {
        "constructor" => "new".to_string(),
        "attr_writer" => name + "=",
        "delegate" => {
            let mut parser = Parser::new();
            let mut cursor = QueryCursor::new();
            parser.set_language(tree_sitter_ruby::language()).unwrap();
            parser.reset();

            let tree = parser.parse(&docs, None).unwrap();
            let query = Query::new(tree_sitter_ruby::language(), DELEGATE_SCHEMA).unwrap();

            let mut matches = cursor.matches(&query, tree.root_node(), docs);

            if let Some(matchy) = matches.next() {
                return if matchy.captures.len() == 2 {
                    let prefix = matchy.captures[1].node.utf8_text(docs).unwrap().to_owned();

                    prefix[1..prefix.len()].to_string() + "_" + &name
                } else {
                    name
                }
            }
            return name
        },
        _ => name.to_string()
    };

    name
}
