use tree_sitter_tags::{Tag, TagsContext, TagsConfiguration};
use std::{fs, str};
use std::fs::File;
use std::io::Write;
use inflector::string::singularize::to_singular;

fn main() {
    let mut context = TagsContext::new();
    let mut tags_file = File::create("tags_output").unwrap();

    // let javascript_config = TagsConfiguration::new(
    //     tree_sitter_javascript::language(),
    //     tree_sitter_javascript::TAGGING_QUERY,
    //     tree_sitter_javascript::LOCALS_QUERY,
    // ).unwrap();

    let ruby_config = TagsConfiguration::new(
        tree_sitter_ruby::language(),
        include_str!("../ruby/tags.scm"),
        tree_sitter_ruby::LOCALS_QUERY,
    ).unwrap();

    let filename = "/Users/nick/Documents/qira/app/models/person.rb";
    let contents = fs::read(filename).unwrap();

    let tags = context.generate_tags(&ruby_config, &contents, None).unwrap().0;

    for tag in tags {
        let tag = tag.unwrap();
        let node_name = ruby_config.syntax_type_name(tag.syntax_type_id);
        let tag_name = &contents[tag.name_range.start..tag.name_range.end];
        let original_name = str::from_utf8(tag_name.clone()).unwrap_or("");

        let name = name_override(&node_name, &original_name, &tag_name);

        tags_file.write(create_tag(&name, &node_name, &tag).as_bytes()).unwrap();

        match node_name {
            "attr_accessor" => {
                tags_file.write(create_tag(&format!("{}=", name), &node_name, &tag).as_bytes()).unwrap();
            },
            "has_many" => {
                tags_file.write(create_tag(&format!("{}=", name), &node_name, &tag).as_bytes()).unwrap();
                tags_file.write(create_tag(&format!("{}_ids", to_singular(&name)), &node_name, &tag).as_bytes()).unwrap();
                tags_file.write(create_tag(&format!("{}_ids=", to_singular(&name)), &node_name, &tag).as_bytes()).unwrap();
            },
            "has_one" | "belongs_to" => {
                tags_file.write(create_tag(&format!("{}=", name), &node_name, &tag).as_bytes()).unwrap();
                tags_file.write(create_tag(&format!("build_{}", name), &node_name, &tag).as_bytes()).unwrap();
                tags_file.write(create_tag(&format!("create_{}", name), &node_name, &tag).as_bytes()).unwrap();
                tags_file.write(create_tag(&format!("create_{}!", name), &node_name, &tag).as_bytes()).unwrap();
            }
            _ => {}
        }
    }
}

fn create_tag<'a>(name: &'a str, node_name: &'a str, tag: &'a Tag) -> String {
    let row = tag.span.start.row;

    let kind = match node_name {
        "method" | "constructor" | "attr_reader" | "attr_writer" | "attr_accessor" => "f",
        "class" => "c",
        "module" => "m",
        "constant" => "C",
        "scope" | "has_many" | "has_one" | "belongs_to" | "singleton_method" => "F",
        _ => node_name
    };


    format!("{}\t{}\t:{}\t{}\n", name, "app/models/person.rb", row + 1, kind)
}

fn name_override<'a>(node_name: &'a str, original_name: &'a str, tag_name: &'a [u8]) -> String {
    let mut name =
        if original_name.chars().nth(0).unwrap() == ':' {
            (&original_name[1..tag_name.len()]).to_string()
        } else {
            original_name.to_string()
        };


    name = match node_name {
        "constructor" => "new".to_string(),
        "attr_writer" => String::from(name) + "=",
        _ => name.to_string()
    };

    name
}
