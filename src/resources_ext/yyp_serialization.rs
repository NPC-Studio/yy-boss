use std::fmt::Write;
pub use yy_typings::{
    texture_group::TextureGroup, AudioGroup, FilesystemPath, ResourceVersion, RoomOrderId, Tags,
    Yyp, YypConfig, YypFolder, YypIncludedFile, YypMetaData, YypResource,
};

const BIG_NUMBER: usize = 2000;
const MEMBER_NUMBER: usize = 70;
const TWO_SPACES: &str = "  ";

#[cfg(target_os = "windows")]
const LINE_ENDING: &str = "\r\n";
#[cfg(not(target_os = "windows"))]
const LINE_ENDING: &str = "\n";

pub trait YypSerialization {
    fn yyp_serialization(&self, indentation: usize) -> String;
}

pub fn serialize_yyp(yyp: &Yyp) -> String {
    let mut output = String::with_capacity(BIG_NUMBER);

    let output_ptr = &mut output;
    print_indentation(output_ptr, 1);
    print_yyp_line(output_ptr, "resourceType", "\"GMProject\"".to_string());
    print_yyp_line(
        output_ptr,
        "resourceVersion",
        yyp.common_data.resource_version.yyp_serialization(1),
    );
    print_yyp_line(
        output_ptr,
        "name",
        yyp.common_data.name.yyp_serialization(1),
    );

    print_yyp_line(
        output_ptr,
        "AudioGroups",
        yyp.audio_groups.yyp_serialization(1),
    );
    print_yyp_line(output_ptr, "configs", yyp.configs.yyp_serialization(1));
    print_yyp_line(
        output_ptr,
        "defaultScriptType",
        yyp.default_script_type.to_string(),
    );
    print_yyp_line(output_ptr, "Folders", yyp.folders.yyp_serialization(1));
    print_yyp_line(
        output_ptr,
        "IncludedFiles",
        yyp.included_files.yyp_serialization(1),
    );
    print_yyp_line(output_ptr, "isEcma", yyp.is_ecma.to_string());

    // note: we don't know what this is, so we just print an empty array
    print_yyp_line(output_ptr, "LibraryEmitters", "[]".to_string());

    print_yyp_line(output_ptr, "MetaData", yyp.meta_data.yyp_serialization(1));

    print_yyp_line(output_ptr, "resources", yyp.resources.yyp_serialization(1));

    print_yyp_line(
        output_ptr,
        "RoomOrderNodes",
        yyp.room_order_nodes.yyp_serialization(1),
    );
    print_yyp_line(
        output_ptr,
        "TextureGroups",
        yyp.texture_groups.yyp_serialization(1),
    );

    // pop off two spaces + the newline
    for _ in 0..TWO_SPACES.len() {
        output_ptr.pop();
    }
    output_ptr.pop();

    format!(
        "{{{line}{output}{line}}}",
        line = LINE_ENDING,
        output = output
    )
}

fn print_yyp_line(string: &mut String, label: &str, value: String) {
    write!(string, "\"{}\": {},{}", label, value, LINE_ENDING).unwrap();
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
        format!(r#"{{"id":{},}}"#, self.id.yyp_serialization(indentation),)
    }
}

impl YypSerialization for YypConfig {
    fn yyp_serialization(&self, mut indentation: usize) -> String {
        fn inner_config_print(string: &mut String, config: &YypConfig, indentation: &mut usize) {
            print_indentation(string, *indentation);
            write!(string, r#"{{"children":["#).unwrap();

            if config.children.is_empty() == false {
                string.push_str(LINE_ENDING);
                // Get us to the write line...
                *indentation += 2;

                for child in config.children.iter() {
                    let old_indentation = *indentation;

                    inner_config_print(string, child, indentation);
                    assert_eq!(
                        old_indentation, *indentation,
                        "Stack on inner children broken"
                    );
                }

                *indentation -= 1;
                print_indentation(string, *indentation);
                string.push_str("],");
                string.push_str(LINE_ENDING);
                print_indentation(string, *indentation);
                write!(string, "\"name\": \"{}\",{}", config.name, LINE_ENDING).unwrap();
                *indentation -= 1;
            } else {
                write!(string, "],\"name\":\"{}\",}},{}", config.name, LINE_ENDING).unwrap();
            }
        }

        let mut output = String::with_capacity(MEMBER_NUMBER);

        // Outer Config
        write!(output, "{{{}", LINE_ENDING).unwrap();
        indentation += 1;
        print_indentation(&mut output, indentation);
        let old_indentation = indentation;

        output.push_str(r#""children": ["#);
        if self.children.is_empty() == false {
            output.push_str(LINE_ENDING);

            indentation += 1;

            for child in self.children.iter() {
                inner_config_print(&mut output, child, &mut indentation);
            }

            indentation -= 1;
            print_indentation(&mut output, indentation);
        }

        print_indentation(&mut output, indentation);
        write!(output, r#""name": "{}","#, self.name).unwrap();
        output.push_str(LINE_ENDING);

        assert_eq!(
            old_indentation, indentation,
            "Child config stack must be balanced"
        );

        write!(output, "],{}", LINE_ENDING).unwrap();
        indentation -= 1;

        assert_eq!(1, indentation, "Stack must be down to 1 indent.");
        print_indentation(&mut output, indentation);
        output.push('}');

        output
    }
}

impl YypSerialization for YypFolder {
    fn yyp_serialization(&self, _: usize) -> String {
        json_trailing_comma(&self)
    }
}

impl YypSerialization for AudioGroup {
    fn yyp_serialization(&self, _: usize) -> String {
        json_trailing_comma(&self)
    }
}

impl YypSerialization for TextureGroup {
    fn yyp_serialization(&self, _: usize) -> String {
        json_trailing_comma(&self).replace("{,}", "{}")
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
            "{{{line}{two}{two}\"IDEVersion\": \"{ide}\",{line}{two}}}",
            two = TWO_SPACES,
            ide = self.ide_version,
            line = LINE_ENDING
        )
    }
}

impl YypSerialization for ResourceVersion {
    fn yyp_serialization(&self, _: usize) -> String {
        serde_json::to_string(self).unwrap()
    }
}

fn json_trailing_comma(t: &impl serde::Serialize) -> String {
    let output = serde_json::to_string(t).unwrap();
    // this is actually peak performance
    output.replace('}', ",}").replace("{,}", "{}")
}

impl<T: YypSerialization> YypSerialization for Vec<T> {
    fn yyp_serialization(&self, mut indentation: usize) -> String {
        if self.is_empty() {
            "[]".to_owned()
        } else {
            let mut output = String::with_capacity(MEMBER_NUMBER);

            write!(output, "[{}", LINE_ENDING).unwrap();
            indentation += 1;

            for value in self.iter() {
                print_indentation(&mut output, indentation);
                write!(
                    output,
                    "{},{}",
                    value.yyp_serialization(indentation),
                    LINE_ENDING
                )
                .unwrap();
            }
            indentation -= 1;

            print_indentation(&mut output, indentation);
            output.push(']');

            output
        }
    }
}

impl YypSerialization for String {
    fn yyp_serialization(&self, _: usize) -> String {
        format!("\"{}\"", self)
    }
}

fn print_indentation(string: &mut String, indentation: usize) {
    for _ in 0..indentation {
        string.push_str(TWO_SPACES);
    }
}

impl YypSerialization for RoomOrderId {
    fn yyp_serialization(&self, i: usize) -> String {
        format!("{{\"roomId\":{},}}", self.room_id.yyp_serialization(i))
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use pretty_assertions::assert_eq as pretty_assert_eq;
//     use std::path::Path;
//     use yy_typings::utils::TrailingCommaUtility;

//     #[test]
//     fn fsystem_path() {
//         let fsystem_path = FilesystemPath {
//             name: "Sprites".to_string(),
//             path: Path::new("folders/Members/Sprites").to_owned(),
//         };

//         pretty_assert_eq!(
//             fsystem_path.yyp_serialization(0),
//             r#"{"name":"Sprites","path":"folders/Members/Sprites",}"#
//         );
//     }

//     #[test]
//     fn yyp_resource() {
//         let yyp_resource = YypResource {
//             id: FilesystemPath {
//                 name: "Sprites".to_string(),
//                 path: Path::new("folders/Members/Sprites.yy").to_owned(),
//             },
//             order: 1,
//         };

//         pretty_assert_eq!(
//             yyp_resource.yyp_serialization(0),
//             r#"{"id":{"name":"Sprites","path":"folders/Members/Sprites.yy",},"order":1,}"#
//         );
//     }

//     #[test]
//     fn yyp() {
//         let yyp = include_str!("../../tests/examples/test_proj/test_proj.yyp");

//         let parse_yyp: Yyp =
//             serde_json::from_str(&TrailingCommaUtility::clear_trailing_comma_once(yyp)).unwrap();
//         let no_mangled_yyp = parse_yyp.yyp_serialization(0);

//         assert_eq!(yyp, no_mangled_yyp);
//     }
// }
