use std::{collections::HashMap, path::{Path, PathBuf}};
use ree_lib::{assets::bundle::Bundle, enums::{EnumMap, load_enum_map}, rsz::RszMap};
use serde::Deserialize;

use crate::save::{game::Game, remap::{Remap, get_asset_paths}};

pub type GameConfigs = HashMap<Game, GamePaths>;

#[derive(Deserialize, Debug, Clone)]
pub struct GamePaths {
    pub rsz: Option<String>,
    pub enums: Option<String>,
    pub enums_raw: Option<String>,
    pub remaps: Option<String>,
    pub bundle: Option<String>,
}

pub struct GameData {
    pub rsz: RszMap,
    pub enums: EnumMap,
    pub remaps: HashMap<String, Remap>,
    pub bundle: Bundle
}

impl GameData {
    pub fn load(paths: &GamePaths, force_raw: bool) -> anyhow::Result<Self> {
        let rsz:    RszMap                 = load_json(paths.rsz.as_ref(), "rsz map")?;
        let enums:  EnumMap                = load_json(paths.enums.as_ref(), "enums")?;
        let remaps: HashMap<String, Remap> = load_json(paths.remaps.as_ref(), "remaps")?;

        let bundle = if force_raw {
            load_bundle_raw(&remaps, &rsz)
        } else {
            load_bundle(paths.bundle.as_deref(), &remaps, &rsz)?
        };

        Ok(Self { rsz, enums, remaps, bundle })
    }
}


pub fn load_game_configs(path: &str) -> anyhow::Result<GameConfigs> {
    let data = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&data)?)
}

fn load_json<T: serde::de::DeserializeOwned + Default>(path: Option<&String>, label: &str) -> anyhow::Result<T> {
    match path {
        Some(p) => Ok(serde_json::from_slice(&std::fs::read(p)?)?),
        None => {
            log::info!("No {} path configured, using default", label);
            Ok(T::default())
        }
    }
}

fn load_enum(path: Option<&String>) -> anyhow::Result<EnumMap> {
    match path {
        Some(p) => Ok(load_enum_map(&PathBuf::from(p))?),
        None => {
            log::info!("No enums path configured, using default");
            Ok(EnumMap::default())
        }
    }
}

fn load_bundle(path: Option<&str>, remaps: &HashMap<String, Remap>, rsz: &RszMap) -> anyhow::Result<Bundle> {
    if let Some(p) = path.map(Path::new).filter(|p| p.exists()) {
        log::info!("Loading bundle from {}", p.display());
        let data = std::fs::read(p)?;
        Ok(bincode::decode_from_slice::<Bundle, _>(&data, bincode::config::standard())?.0)
    } else {
        Ok(load_bundle_raw(remaps, rsz))
    }
}

fn load_bundle_raw(remaps: &HashMap<String, Remap>, rsz: &RszMap) -> Bundle {
    log::info!("Loading assets from raw remap paths");
    let mut bundle = Bundle::default();
    bundle.load_from_paths(get_asset_paths(remaps), rsz);
    bundle
}

