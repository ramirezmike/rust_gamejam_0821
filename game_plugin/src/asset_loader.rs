use bevy::{prelude::*,};
use bevy::asset::{AssetLoader, LoadContext, LoadedAsset};
use bevy::reflect::{TypeUuid};
use bevy::utils::{BoxedFuture};
use serde::Deserialize;

use crate::{level_collision, cutscene, enemy};


// this is for hot reloading
#[derive(Default)]
pub struct LevelsAssetLoader;
impl AssetLoader for LevelsAssetLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            println!("Level asset reloaded");
            let lvl_asset = ron::de::from_bytes::<LevelInfo>(bytes)?;
            load_context.set_default_asset(LoadedAsset::new(lvl_asset));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["lvl", ]
    }
}

#[derive(Default)]
pub struct AssetsLoading {
    pub asset_handles: Vec<HandleUntyped>
}
pub fn check_assets_ready(
    mut state: ResMut<State<crate::AppState>>,
    server: Res<AssetServer>,
    loading: Res<AssetsLoading>,
) {
    println!("Loading...");
    use bevy::asset::LoadState;

    let mut ready = true;

    for handle in loading.asset_handles.iter() {
        match server.get_load_state(handle) {
            LoadState::Failed => {
                // one of our assets had an error
            }
            LoadState::Loaded => {
            }
            _ => {
                ready = false;
            }
        }
    }

    if ready {
        state.set(crate::AppState::MainMenu).unwrap();
    }
}

#[derive(Default)]
pub struct LevelInfoState {
    pub handle: Handle<LevelInfo>,
}
#[derive(Debug, Clone, Deserialize, TypeUuid)]
#[uuid = "39cadc56-aa9c-4543-8640-a018b74b5052"]
pub struct LevelInfo {
    pub collision_info: level_collision::LevelCollisionInfo,
    pub cutscenes: cutscene::Cutscenes,
    pub enemies: Vec::<enemy::EnemySpawnPoint>,
}

