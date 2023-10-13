#[macro_export]
// Default logic for generating tags
macro_rules! default_generate_tags {
    () => {
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

                vec![create_tag(original_name, node_name, &tag, filename)]
            })
            .collect::<Vec<Tag>>()
            }
    }
}
