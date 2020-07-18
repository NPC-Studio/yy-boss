pub use super::yy_typings::{
    texture_group::TextureGroup, AudioGroup, FilesystemPath, Tags, Yyp, YypConfig, YypFolder,
    YypIncludedFile, YypMetaData, YypResource,
};

const BIG_NUMBER: usize = 2000;
const MEMBER_NUMBER: usize = 70;
const TWO_SPACES: &'static str = "  ";

pub trait YypSerialization {
    fn yyp_serialization(&self, indentation: usize) -> String;
}

impl YypSerialization for Yyp {
    fn yyp_serialization(&self, _: usize) -> String {
        let mut output = String::with_capacity(BIG_NUMBER);

        let output_ptr = &mut output;
        print_indentation(output_ptr, 1);
        print_yyp_line(output_ptr, "resources", self.resources.yyp_serialization(1));
        print_yyp_line(output_ptr, "Options", self.options.yyp_serialization(1));
        print_yyp_line(output_ptr, "isDnDProject", self.is_dn_d_project.to_string());
        print_yyp_line(output_ptr, "isEcma", self.is_ecma.to_string());
        // We need to do this here because Rust doesn't like empty strings
        if self.tutorial_path.is_empty() {
            output_ptr.push_str("\"tutorialPath\": \"\",\r\n");
            print_indentation(output_ptr, 1);
        } else {
            print_yyp_line(output_ptr, "tutorialPath", self.tutorial_path.to_string());
        }

        print_yyp_line(output_ptr, "configs", self.configs.yyp_serialization(1));
        print_yyp_line(
            output_ptr,
            "RoomOrder",
            self.room_order.yyp_serialization(1),
        );
        print_yyp_line(output_ptr, "Folders", self.folders.yyp_serialization(1));
        print_yyp_line(
            output_ptr,
            "AudioGroups",
            self.audio_groups.yyp_serialization(1),
        );
        print_yyp_line(
            output_ptr,
            "TextureGroups",
            self.texture_groups.yyp_serialization(1),
        );
        print_yyp_line(
            output_ptr,
            "IncludedFiles",
            self.included_files.yyp_serialization(1),
        );
        print_yyp_line(output_ptr, "MetaData", self.meta_data.yyp_serialization(1));
        print_yyp_line(
            output_ptr,
            "resourceVersion",
            self.resource_version.yyp_serialization(1),
        );
        print_yyp_line(output_ptr, "name", self.name.yyp_serialization(1));
        print_yyp_line(output_ptr, "tags", self.tags.yyp_serialization(1));
        output_ptr.push_str("\"resourceType\": \"GMProject\",");

        format!("{{\r\n{}\r\n}}", output)
    }
}

fn print_yyp_line(string: &mut String, label: &str, value: String) {
    string.push_str(&format!("\"{}\": {},\r\n", label, value));
    print_indentation(string, 1);
}

impl YypSerialization for FilesystemPath {
    fn yyp_serialization(&self, _: usize) -> String {
        format!(
            r#"{{"name":"{}","path":"{}",}}"#,
            self.name,
            self.path.to_string_lossy(),
        )
    }
}

impl YypSerialization for YypResource {
    fn yyp_serialization(&self, indentation: usize) -> String {
        format!(
            r#"{{"id":{},"order":{},}}"#,
            self.id.yyp_serialization(indentation),
            self.order
        )
    }
}

impl YypSerialization for YypConfig {
    fn yyp_serialization(&self, mut indentation: usize) -> String {
        fn inner_config_print(string: &mut String, config: &YypConfig, indentation: &mut usize) {
            print_indentation(string, *indentation);
            string.push_str(&format!(r#"{{"name":"{}","children":["#, config.name));

            if config.children.is_empty() == false {
                // Get us to the write line...
                *indentation += 2;
                string.push_str("\r\n");

                for child in config.children.iter() {
                    println!("Config child: {:#?}", child);
                    let old_indentation = *indentation;

                    inner_config_print(string, child, indentation);
                    assert_eq!(
                        old_indentation, *indentation,
                        "Stack on inner children broken"
                    );
                }

                *indentation -= 1;
                print_indentation(string, *indentation);
                string.push_str("],},");
                string.push_str("\r\n");
                *indentation -= 1;
            } else {
                string.push_str("],},");
                string.push_str("\r\n");
            }
        }

        let mut output = String::with_capacity(MEMBER_NUMBER);

        // Outer Config
        output.push_str("{\r\n");
        indentation += 1;
        print_indentation(&mut output, indentation);
        output.push_str(&format!(r#""name": "{}","#, self.name));

        output.push_str("\r\n");
        print_indentation(&mut output, indentation);
        let old_indentation = indentation;

        output.push_str(&format!(r#""children": ["#));
        if self.children.is_empty() == false {
            output.push_str("\r\n");

            indentation += 1;

            for child in self.children.iter() {
                inner_config_print(&mut output, child, &mut indentation);
            }

            indentation -= 1;
            print_indentation(&mut output, indentation);
        }

        assert_eq!(
            old_indentation, indentation,
            "Child config stack must be balanced"
        );

        output.push_str("],\r\n");
        indentation -= 1;

        assert_eq!(1, indentation, "Stack must be down to 1 indent.");
        print_indentation(&mut output, indentation);
        output.push('}');

        output
    }
}

impl YypSerialization for YypFolder {
    fn yyp_serialization(&self, indentation: usize) -> String {
        format!(
            r#"{{"folderPath":"{}","order":{},"resourceVersion":"{}","name":"{}","tags":{},"resourceType":"GMFolder",}}"#,
            self.folder_path.inner(),
            self.order,
            self.resource_version,
            self.name,
            self.tags.yyp_serialization(indentation)
        )
    }
}

impl YypSerialization for AudioGroup {
    fn yyp_serialization(&self, _: usize) -> String {
        json_trailing_comma(&self)
    }
}

impl YypSerialization for TextureGroup {
    fn yyp_serialization(&self, _: usize) -> String {
        json_trailing_comma(&self)
    }
}

impl YypSerialization for YypIncludedFile {
    fn yyp_serialization(&self, _: usize) -> String {
        json_trailing_comma(&self)
    }
}

impl YypSerialization for YypMetaData {
    fn yyp_serialization(&self, _: usize) -> String {
        format!(
            "{{\r\n{}{}\"IDEVersion\": \"{}\",\r\n{}}}",
            TWO_SPACES, TWO_SPACES, self.ide_version, TWO_SPACES
        )
    }
}

fn json_trailing_comma(t: &impl serde::Serialize) -> String {
    let mut output = serde_json::to_string(t).unwrap();
    output.truncate(output.len() - 1);
    output.push_str(",}");

    output
}

impl<T: YypSerialization> YypSerialization for Vec<T> {
    fn yyp_serialization(&self, mut indentation: usize) -> String {
        if self.is_empty() {
            format!("[]")
        } else {
            let mut output = String::with_capacity(MEMBER_NUMBER);

            output.push_str("[\r\n");
            indentation += 1;

            for value in self.iter() {
                print_indentation(&mut output, indentation);
                output.push_str(&format!("{},\r\n", value.yyp_serialization(indentation)));
            }
            indentation -= 1;

            print_indentation(&mut output, indentation);
            output.push_str("]");

            output
        }
    }
}

impl YypSerialization for String {
    fn yyp_serialization(&self, _: usize) -> String {
        format!("\"{}\"", self.to_string())
    }
}

fn print_indentation(string: &mut String, indentation: usize) {
    for _ in 0..indentation {
        string.push_str(TWO_SPACES);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq as pretty_assert_eq;
    use std::path::Path;
    use yy_typings::utils::TrailingCommaUtility;

    #[test]
    fn fsystem_path() {
        let fsystem_path = FilesystemPath {
            name: "Sprites".to_string(),
            path: Path::new("folders/Members/Sprites").to_owned(),
        };

        pretty_assert_eq!(
            fsystem_path.yyp_serialization(0),
            r#"{"name":"Sprites","path":"folders/Members/Sprites",}"#
        );
    }

    #[test]
    fn yyp_resource() {
        let yyp_resource = YypResource {
            id: FilesystemPath {
                name: "Sprites".to_string(),
                path: Path::new("folders/Members/Sprites.yy").to_owned(),
            },
            order: 1,
        };

        pretty_assert_eq!(
            yyp_resource.yyp_serialization(0),
            r#"{"id":{"name":"Sprites","path":"folders/Members/Sprites.yy",},"order":1,}"#
        );
    }

    #[test]
    fn yyp() {
        let yyp = include_str!("../../tests/examples/test_proj/test_proj.yyp");

        let parse_yyp: Yyp =
            serde_json::from_str(&TrailingCommaUtility::clear_trailing_comma_once(yyp)).unwrap();
        let no_mangled_yyp = parse_yyp.yyp_serialization(0);

        assert_eq!(yyp, no_mangled_yyp);
    }
}
