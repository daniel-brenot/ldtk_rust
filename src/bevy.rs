// Code for bevy asset loader

use std::path::{Path, PathBuf};

use bevy::{asset::{io::Reader, Asset, AssetLoader, AssetPath, Handle, LoadedAsset}, image::Image, platform::collections::HashMap, reflect::TypePath};

use crate::{LdtkError, Level};

#[derive(Default, Clone, Debug)]
pub struct LdtkPlugin();

impl Plugin for LdtkPlugin {
    fn build(&self, mut app: &mut App) {
        app
            .add_asset::<BlobAsset>()
            .add_asset::<LdtkMap>()
            .add_asset_loader(LdtkLoader);
    }
}


#[derive(TypePath, Asset)]
pub struct BlobAsset {
    pub data: Vec<u8>,
}

impl Level {
    pub async fn new_with_context(f: &Path, load_context: &mut bevy::asset::LoadContext<'_>) -> Result<Self, LdtkError> {
        let asset: LoadedAsset<BlobAsset> = load_context.loader().immediate().load(f).await?;
        let o: Level = serde_json::from_reader(asset.get().data.as_slice())?;
        Ok(o)
    }
}

#[derive(TypePath, Asset)]
pub struct LdtkMap {
    pub project: crate::Project,
    pub tilesets: HashMap<i64, Handle<Image>>,
}

pub struct LdtkLoader;

impl AssetLoader for LdtkLoader {
    type Asset = LdtkMap;
    type Settings = ();
    type Error = LdtkError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        load_context: &mut bevy::asset::LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let path = load_context.asset_path().path().to_path_buf();
        let mut project = crate::Project::from_slice(&bytes)?;

        if project.external_levels {
            // get all the file names
            let mut all_level_files: Vec<PathBuf> = Vec::new();
            for level in project.levels.iter_mut() {
                let level_file_path = level.external_rel_path.as_ref().ok_or(LdtkError::ExternalLevelNameNotFoundError)?;
                all_level_files.push(level_file_path.into());
            }

            // get rid of existing levels (which don't have much data)
            project.clear_levels();

            let parent = path
                    .parent()
                    .ok_or(LdtkError::PathToStringError())?
                    .to_str()
                    .ok_or(LdtkError::PathToStringError())?;

            // now add each of them to our struct
            for file in all_level_files.iter() {
                let mut full_path = PathBuf::new();
                
                let mf = file.to_str()
                    .ok_or(LdtkError::PathToStringError())?;
                full_path.push(format!("{parent}/{mf}"));
                let level_ldtk = crate::Level::new_with_context(&full_path, load_context).await?;
                project.levels.push(level_ldtk);
            }
        }
        
        let dependencies: Vec<(i64, AssetPath)> = project
            .defs
            .tilesets
            .iter()
            .filter_map(|tileset| {
                tileset.rel_path.as_ref().map(|rel_path| {
                    (
                        tileset.uid,
                        load_context.path().parent().unwrap().join(rel_path).into(),
                    )
                })
            })
            .collect();

        let ldtk_map = LdtkMap {
            project,
            tilesets: dependencies
                .iter()
                .map(|dep| (dep.0, load_context.load(dep.1.clone())))
                .collect(),
        };
        Ok(ldtk_map)
    }

    fn extensions(&self) -> &[&str] {
        static EXTENSIONS: &[&str] = &["ldtk"];
        EXTENSIONS
    }
}