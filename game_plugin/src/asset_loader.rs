use bevy::{prelude::*,};
use bevy::asset::{AssetLoader, LoadContext, LoadedAsset};
use bevy::reflect::{TypeUuid};
use bevy::utils::{BoxedFuture};
use serde::Deserialize;


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
//            let custom_asset = ron::de::from_bytes::<LevelsAsset>(bytes)?;
//            load_context.set_default_asset(LoadedAsset::new(custom_asset));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["custom"]
    }
}

#[derive(Default)]
pub struct AssetsLoading(Vec<HandleUntyped>);
pub fn check_assets_ready(
    mut state: ResMut<State<crate::AppState>>,
    server: Res<AssetServer>,
    loading: Res<AssetsLoading>,
) {
    println!("Loading...");
    use bevy::asset::LoadState;

    let mut ready = true;

    for handle in loading.0.iter() {
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
