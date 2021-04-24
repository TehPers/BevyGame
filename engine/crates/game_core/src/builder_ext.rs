use game_lib::bevy::{
    ecs::{self as bevy_ecs, component::Component, schedule::RunCriteriaDescriptor},
    prelude::*,
};
use std::{fmt::Debug, hash::Hash};
use strum::{EnumIter, IntoEnumIterator};

use crate::{
    modes::{Mode, ModeSystem},
    ModeStage,
};

pub trait AppBuilderExt {
    fn add_mode<M>(&mut self, initial: M) -> &mut Self
    where
        M: Component + Debug + Eq;

    fn add_state_events_to_stage<S, L>(&mut self, stage: L, state: S) -> &mut Self
    where
        S: Component + Debug + Clone + Eq + Hash,
        L: StageLabel;

    fn add_state_events<S>(&mut self, state: S) -> &mut Self
    where
        S: Component + Debug + Clone + Eq + Hash,
    {
        self.add_state_events_to_stage(CoreStage::Update, state)
    }

    fn add_many_state_events_to_stage<S, L, I>(&mut self, stage: L, states: I) -> &mut Self
    where
        S: Component + Debug + Clone + Eq + Hash,
        I: IntoIterator<Item = S>,
        L: StageLabel;

    fn add_many_state_events<S, I>(&mut self, states: I) -> &mut Self
    where
        S: Component + Debug + Clone + Eq + Hash,
        I: IntoIterator<Item = S>,
    {
        self.add_many_state_events_to_stage(CoreStage::Update, states)
    }
}

impl AppBuilderExt for AppBuilder {
    fn add_mode<M>(&mut self, initial: M) -> &mut Self
    where
        M: Component + Debug + Eq,
    {
        self.insert_resource(Mode::new(initial))
            .add_system_to_stage(
                ModeStage::UpdateModes,
                crate::modes::update_mode::<M>
                    .system()
                    .label(ModeSystem::UpdateModes)
                    .in_ambiguity_set(ModeSystem::UpdateModes),
            )
    }

    fn add_state_events_to_stage<S, L>(&mut self, stage: L, state: S) -> &mut Self
    where
        S: Component + Debug + Clone + Eq + Hash,
        L: StageLabel,
    {
        self.stage(stage, |stage: &mut SystemStage| {
            for event in ModeEvent::iter() {
                stage.add_system_run_criteria(
                    event
                        .run_criteria(state.clone())
                        .label_discard_if_duplicate(StateEventLabel(state.clone(), event)),
                );
            }

            stage
        })
    }

    fn add_many_state_events_to_stage<S, L, I>(&mut self, stage: L, states: I) -> &mut Self
    where
        S: Component + Debug + Clone + Eq + Hash,
        I: IntoIterator<Item = S>,
        L: StageLabel,
    {
        self.stage(stage, move |stage: &mut SystemStage| {
            for state in states.into_iter() {
                for event in ModeEvent::iter() {
                    stage.add_system_run_criteria(
                        event
                            .run_criteria(state.clone())
                            .label_discard_if_duplicate(StateEventLabel(state.clone(), event)),
                    );
                }
            }

            stage
        })
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, EnumIter)]
pub enum ModeEvent {
    /// Runs every tick that a mode is active.
    Active,

    /// Runs every tick that a mode is inactive.
    Inactive,

    /// Runs only on the tick after a mode was entered.
    Enter,

    /// Runs only on the tick after a mode was exited.
    Exit,
}

impl ModeEvent {
    pub fn run_criteria<S>(self, state: S) -> RunCriteriaDescriptor
    where
        S: Component + Debug + Clone + Eq + Hash,
    {
        match self {
            ModeEvent::Active => State::on_update(state),
            ModeEvent::Inactive => State::on_inactive_update(state),
            ModeEvent::Enter => State::on_enter(state),
            ModeEvent::Exit => State::on_exit(state),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, RunCriteriaLabel)]
pub struct StateEventLabel<S>(pub S, pub ModeEvent)
where
    S: Component + Debug + Clone + Eq + Hash;
