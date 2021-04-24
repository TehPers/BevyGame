use game_lib::bevy::prelude::*;

#[derive(Debug, Default)]
pub struct RequiredAssets {
    pub loading_assets: Vec<HandleUntyped>,
}
