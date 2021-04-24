use crate::{modes::Mode, ModeEvent};
use game_lib::bevy::{
    ecs::{component::Component, schedule::ShouldRun, system::BoxedSystem},
    prelude::*,
};
use std::fmt::Debug;

pub trait ModeExt: Sized + Clone + Debug + Component + Eq {
    /// Run criteria for when an event occurs on this state
    fn on(self, event: ModeEvent) -> BoxedSystem<(), ShouldRun> {
        match event {
            ModeEvent::Active => Mode::if_active(self),
            ModeEvent::Inactive => Mode::if_inactive(self),
            ModeEvent::Enter => Mode::if_entered(self),
            ModeEvent::Exit => Mode::if_exited(self),
        }
    }

    /// Creates a system which transitions to this state
    fn transition_system(self) -> BoxedSystem<(), ()> {
        Box::new(
            (|mut main_loading_state: ResMut<Mode<Self>>, target: Local<Option<Self>>| {
                let target = target.as_ref().unwrap().clone();
                match main_loading_state.enqueue(target) {
                    Ok(_) | Err(crate::modes::ModeEnqueueError::ModeAlreadySet { .. }) => {}
                    Err(error) => Err(error).unwrap(),
                }
            })
            .system()
            .config(|(_, target)| *target = Some(Some(self))),
        )
    }
}

impl<T> ModeExt for T where T: Sized + Clone + Debug + Component + Eq {}
