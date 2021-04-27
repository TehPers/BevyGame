use game_lib::bevy::{
    asset::{Asset, AssetPath},
    ecs::{self as bevy_ecs, system::SystemParam},
    prelude::*,
};

#[derive(Debug, Default)]
pub struct RequiredAssets {
    pub loading_assets: Vec<HandleUntyped>,
}

#[derive(SystemParam)]
pub struct RequiredAssetLoader<'a> {
    pub asset_server: Res<'a, AssetServer>,
    pub required_assets: ResMut<'a, RequiredAssets>,
}

impl<'a> RequiredAssetLoader<'a> {
    pub fn load_required<'b, T, P>(&mut self, path: P) -> Handle<T>
    where
        T: Asset,
        P: Into<AssetPath<'a>>,
    {
        let handle = self.asset_server.load(path);
        self.required_assets
            .loading_assets
            .push(handle.clone_untyped());
        handle
    }
}
