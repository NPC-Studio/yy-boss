use serde::{Deserialize, Serialize};
use yy_typings::{ResourceVersion, ViewPath};

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectMetadata {
    pub name: String,
    pub ide_version: String,
    pub yyp_version: ResourceVersion,
    pub root_file: ViewPath,
}
