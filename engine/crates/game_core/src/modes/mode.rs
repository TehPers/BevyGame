use game_lib::{
    bevy::{
        ecs::{self as bevy_ecs, component::Component, schedule::ShouldRun, system::BoxedSystem},
        prelude::*,
    },
    derive_more::{Display, Error},
};
use std::fmt::Debug;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
enum ModeState {
    Started,
    Changed,
    Unchanged,
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct Mode<T: Component + Eq> {
    current: T,
    queued: Option<T>,
    prev: Option<T>,
    state: ModeState,
}

impl<T: Component + Debug + Eq> Mode<T> {
    pub fn new(initial: T) -> Self {
        Mode {
            current: initial,
            queued: None,
            prev: None,
            state: ModeState::Started,
        }
    }

    pub fn get(&self) -> &T {
        &self.current
    }

    pub fn get_prev(&self) -> Option<&T> {
        self.prev.as_ref()
    }

    pub fn get_queued(&self) -> Option<&T> {
        self.queued.as_ref()
    }

    pub fn enqueue(&mut self, value: T) -> Result<(), ModeEnqueueError<T>> {
        if self.queued.is_some() {
            Err(ModeEnqueueError::ChangeAlreadyQueued { value })
        } else if self.current == value {
            Err(ModeEnqueueError::ModeAlreadySet { value })
        } else {
            self.queued = Some(value);
            Ok(())
        }
    }

    pub fn update(&mut self) {
        let mut changed = self.state == ModeState::Started;
        if let Some(queued) = self.queued.take() {
            let prev = std::mem::replace(&mut self.current, queued);
            self.prev = Some(prev);
            changed = true;
        } else {
            self.prev = None;
        }

        self.state = if changed {
            ModeState::Changed
        } else {
            ModeState::Unchanged
        };
    }

    pub fn if_active(value: T) -> BoxedSystem<(), ShouldRun> {
        #[derive(Clone, Copy, PartialEq, Eq)]
        enum DelayState {
            Delayed,
            Ready,
        }

        struct LocalState<T> {
            target: T,
            delay: DelayState,
        }

        Box::new(
            (|mode: ResMut<Mode<T>>, mut state: Local<Option<LocalState<T>>>| -> ShouldRun {
                let state = state.as_mut().unwrap();
                if mode.get() == &state.target {
                    match state.delay {
                        DelayState::Ready if mode.state == ModeState::Changed => {
                            state.delay = DelayState::Delayed;
                            ShouldRun::NoAndCheckAgain
                        }
                        DelayState::Delayed | DelayState::Ready => {
                            state.delay = DelayState::Ready;
                            ShouldRun::Yes
                        }
                    }
                } else {
                    ShouldRun::No
                }
            })
            .system()
            .config(|(_, state)| {
                *state = Some(Some(LocalState {
                    target: value,
                    delay: DelayState::Ready,
                }))
            }),
        )
    }

    pub fn if_inactive(value: T) -> BoxedSystem<(), ShouldRun> {
        #[derive(Clone, Copy, PartialEq, Eq)]
        enum DelayState {
            Delayed,
            Ready,
        }

        struct LocalState<T> {
            target: T,
            delay: DelayState,
        }

        Box::new(
            (|mode: ResMut<Mode<T>>, mut state: Local<Option<LocalState<T>>>| -> ShouldRun {
                let state = state.as_mut().unwrap();
                if mode.get() != &state.target {
                    match state.delay {
                        DelayState::Ready if mode.state == ModeState::Changed => {
                            state.delay = DelayState::Delayed;
                            ShouldRun::NoAndCheckAgain
                        }
                        DelayState::Delayed | DelayState::Ready => {
                            state.delay = DelayState::Ready;
                            ShouldRun::Yes
                        }
                    }
                } else {
                    ShouldRun::No
                }
            })
            .system()
            .config(|(_, state)| {
                *state = Some(Some(LocalState {
                    target: value,
                    delay: DelayState::Ready,
                }))
            }),
        )
    }

    pub fn if_entered(value: T) -> BoxedSystem<(), ShouldRun> {
        Box::new(
            (|mode: ResMut<Mode<T>>, target: Local<Option<T>>| -> ShouldRun {
                target
                    .as_ref()
                    .filter(|&target| mode.state == ModeState::Changed && mode.get() == target)
                    .map(|_| ShouldRun::Yes)
                    .unwrap_or(ShouldRun::No)
            })
            .system()
            .config(|(_, target)| *target = Some(Some(value))),
        )
    }

    pub fn if_exited(value: T) -> BoxedSystem<(), ShouldRun> {
        Box::new(
            (|mode: ResMut<Mode<T>>, target: Local<Option<T>>| -> ShouldRun {
                target
                    .as_ref()
                    .filter(|&target| mode.state == ModeState::Changed && mode.get() != target)
                    .and_then(|target| mode.get_prev().filter(|&prev| prev == target))
                    .map(|_| ShouldRun::Yes)
                    .unwrap_or(ShouldRun::No)
            })
            .system()
            .config(|(_, target)| *target = Some(Some(value))),
        )
    }
}

#[derive(Debug, Display, Error)]
pub enum ModeEnqueueError<T>
where
    T: Component + Eq,
{
    #[display(fmt = "a change was already queued")]
    ChangeAlreadyQueued { value: T },

    #[display(fmt = "the requested mode is already the current mode")]
    ModeAlreadySet { value: T },
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, SystemLabel, AmbiguitySetLabel)]
pub enum ModeSystem {
    UpdateModes,
}
