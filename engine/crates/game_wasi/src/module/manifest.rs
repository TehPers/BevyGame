use std::path::PathBuf;

use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ModuleManifest {
    pub id: String,
    pub version: Version,
    pub entry: PathBuf,
    #[serde(default)]
    pub dependencies: Vec<ModuleDependency>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ModuleDependency {
    pub id: String,
    pub versions: VersionReq,
    #[serde(default)]
    pub optional: bool,
}
