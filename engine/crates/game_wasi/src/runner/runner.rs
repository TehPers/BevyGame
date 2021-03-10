use crate::module::ModuleManifest;
use anyhow::{anyhow, bail, Context};
use bevy::utils::{HashMap, HashSet};
use std::{
    fmt::{Debug, Formatter},
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};
use wasmer::{
    import_namespace, ExportError, Instance, Memory, MemoryType, Module, Pages, Store, Val,
};
use wasmer_wasi::WasiState;

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct ModuleInfo {
    pub manifest: ModuleManifest,
    pub entry_dir: PathBuf,
    pub entry_path: PathBuf,
    pub data_path: PathBuf,
}

#[derive(Clone)]
struct ActiveModule {
    info: ModuleInfo,
    instance: Instance,
}

impl Debug for ActiveModule {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ActiveModule")
            .field("info", &self.info)
            .finish()
    }
}

pub struct WasmRunner {
    shared_memory: Memory,
    active_modules: HashMap<String, ActiveModule>,
}

impl WasmRunner {
    pub fn new(bin_path: &Path) -> anyhow::Result<Self> {
        let mod_path: PathBuf = [bin_path, Path::new("mods")].iter().collect();
        let mod_data_path: PathBuf = [bin_path, Path::new("mod_data")].iter().collect();

        // Find modules
        let mut modules = HashMap::default();
        for dir in mod_path.read_dir()? {
            let dir = dir?;
            let dir_path = dir.path();

            // Load manifest
            let manifest_path: PathBuf = {
                let mut path = dir_path.clone();
                path.push("manifest.json");
                path
            };
            let manifest_file = File::open(manifest_path)?;
            let manifest: ModuleManifest = serde_json::from_reader(BufReader::new(manifest_file))?;

            // Verify entry module is in the same directory
            let entry_path: PathBuf = [&dir_path, &manifest.entry].iter().collect();
            let entry_path = entry_path.canonicalize()?;
            let data_path: PathBuf = match entry_path.strip_prefix(&dir_path) {
                Ok(remainder) => [&mod_data_path, remainder].iter().collect(),
                Err(_) => {
                    bail!(
                        "entry for {} must be contained in its directory ({})",
                        manifest.id,
                        dir_path.display()
                    )
                }
            };

            modules.insert(
                manifest.id.clone(),
                ModuleInfo {
                    manifest,
                    entry_dir: dir_path,
                    entry_path,
                    data_path,
                },
            );
        }

        let mut load_order = Vec::new();
        let mut sorted = HashSet::default();
        let mut unsorted: HashSet<_> = modules.keys().map(|s| s.as_str()).collect();
        while !unsorted.is_empty() {
            let next = unsorted
                .iter()
                .find_map(|&id| {
                    modules.get(id).filter(|info| {
                        info.manifest
                            .dependencies
                            .iter()
                            .all(|dependency| sorted.contains(dependency.id.as_str()))
                    })
                })
                .ok_or_else(|| anyhow!("could not resolve module load order"))?;

            unsorted.remove(next.manifest.id.as_str());
            sorted.insert(next.manifest.id.as_str());
            load_order.push(next);
        }

        // Load modules
        // TODO: make this multithreaded
        let store = Store::default();
        let shared_memory = Memory::new(&store, MemoryType::new(Pages(1024), None, true))
            .context("failure creating shared memory for modules")?;
        let active_modules = HashMap::default();
        for module_info in load_order {
            let entry_bytes = std::fs::read(&module_info.entry_path)?;
            let module = Module::new(&store, &entry_bytes)?;
            let mut wasi_env = WasiState::new("mod_core")
                .preopen(|p| {
                    p.directory(&module_info.entry_dir)
                        .read(true)
                        .write(false)
                        .create(false)
                        .alias("/mod")
                })?
                .preopen(|p| {
                    p.directory(&module_info.data_path)
                        .read(true)
                        .write(true)
                        .create(true)
                        .alias("/data")
                })?
                .env("PWD", "/")
                .finalize()?;
            let mut imports = wasi_env.import_object(&module)?;
            let env_namespace = import_namespace! {
                {
                    "memory" => shared_memory.clone(),
                }
            };
            imports.register("env", env_namespace);
            let instance = Instance::new(&module, &imports)?;

            // Call _start function by convention
            if let Ok(function) = instance.exports.get_native_function::<(), ()>("_start") {
                function.call()?;
            }
        }

        Ok(WasmRunner {
            shared_memory,
            active_modules,
        })
    }

    pub fn on_update(&mut self, params: &[Val]) -> anyhow::Result<()> {
        self.call_all_if_exists("on_update", params)
            .into_iter()
            .map(|(_, result)| result.map(|_| ()))
            .collect()
    }

    pub fn call_all_if_exists(
        &mut self,
        function_name: &str,
        params: &[Val],
    ) -> HashMap<&ModuleInfo, anyhow::Result<Option<Box<[Val]>>>> {
        self.active_modules
            .iter()
            .map(|(_id, active_module)| {
                let result = match active_module.instance.exports.get_function(function_name) {
                    Ok(function) => function
                        .call(params)
                        .context("error during function execution")
                        .map(|result| Some(result)),
                    Err(ExportError::Missing(_)) => Ok(None),
                    Err(error) => Err(error).context("error getting exported function"),
                };
                (&active_module.info, result)
            })
            .collect()
    }
}
