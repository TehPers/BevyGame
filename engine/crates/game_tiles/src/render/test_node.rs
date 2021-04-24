use crate::render::RegionData;
use game_lib::{
    bevy::{
        ecs::system::BoxedSystem,
        prelude::*,
        render::{
            render_graph::{Node, SystemNode},
            texture::TEXTURE_ASSET_INDEX,
        },
    },
    crossbeam::channel::{Receiver, Sender},
    tracing::{self, instrument},
};

type Command = ();

pub struct TestNode {
    sender: Sender<Command>,
    receiver: Receiver<Command>,
}

impl TestNode {
    pub fn new() -> Self {
        let (sender, receiver) = game_lib::crossbeam::channel::unbounded();
        TestNode { sender, receiver }
    }
}

impl SystemNode for TestNode {
    fn get_system(&self) -> BoxedSystem {
        Box::new(
            test_node_system
                .system()
                .config(|(config, _)| *config = Some(Some(self.sender.clone()))),
        )
    }
}

impl Node for TestNode {
    #[instrument(skip(self, world, render_context, input, output))]
    fn update(
        &mut self,
        world: &World,
        render_context: &mut dyn game_lib::bevy::render::renderer::RenderContext,
        input: &game_lib::bevy::render::render_graph::ResourceSlots,
        output: &mut game_lib::bevy::render::render_graph::ResourceSlots,
    ) {
        output.set(0, todo!());
    }
}

#[instrument(skip(state, queries))]
fn test_node_system(
    mut state: Local<Option<Sender<Command>>>,
    mut queries: QuerySet<(
        Query<(&Handle<Texture>, &mut RenderPipelines), Changed<Handle<RegionData>>>,
        Query<&mut RenderPipelines, With<Handle<RegionData>>>,
    )>,
) {
    for pipeline in queries.q1_mut().iter_mut() {}
}
