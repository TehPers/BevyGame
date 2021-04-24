use game_lib::serde::{Deserialize, Serialize};
use semver::{Version, VersionReq};
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(crate = "game_lib::serde")]
pub struct ModuleManifest {
    pub id: String,
    pub version: Version,
    pub entry: PathBuf,
    #[serde(default)]
    pub dependencies: Vec<ModuleDependency>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(crate = "game_lib::serde")]
pub struct ModuleDependency {
    pub id: String,
    pub versions: VersionReq,
    #[serde(default)]
    pub optional: bool,
}
