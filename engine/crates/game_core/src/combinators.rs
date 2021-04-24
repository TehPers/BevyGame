//! Defines several run criteria combinators for bevy 0.5. There are four
//! combinators defined in this file:
//!  - Simple piping combinators ([`not`]): Allows a run criteria to be negated
//!    via the use of chains or labels and pipes.
//!  - Piping combinators ([`and`], [`or`]): Allows two run criteria to be
//!    combined via the use of chains or labels and pipes. To use these, define
//!    your preceeding run criteria on your stage via
//!    [`Stage::add_system_run_criteria()`] or
//!    [`Stage::with_system_run_criteria()`] and label it, then use the
//!    combinator by wrapping your second run criteria with it and passing it
//!    to [`RunCriteria::pipe()`] along with the label for your preceeding
//!    criteria. Alternatively, use [`IntoChainSystem::chain()`].
//!  - Compound combinators ([`if_all`], [`if_any`]): Allows many run criteria
//!    to be combined, however does not support run criteria labels.
//!
//! All combinators are short-circuiting, meaning they are not executed unless
//! they are needed to in order to determine if a system should run.
//!
//! [`RunCriteria::pipe()`]: game_lib::bevy::ecs::schedule::RunCriteria

use game_lib::bevy::{
    ecs::{
        archetype::{Archetype, ArchetypeComponentId},
        component::ComponentId,
        query::Access,
        schedule::ShouldRun,
        system::{BoxedSystem, SystemId},
    },
    prelude::*,
};
use std::borrow::Cow;

/// Identity system. Accepts any value as input and immediately returns it as
/// output. Useful for chaining together systems.
pub fn identity<T>(input: In<T>) -> T {
    input.0
}

macro_rules! impl_system_for_combinator {
    (
        $struct_name:ident $(<$($type_arg:tt),*>)? $((where $(($($type_bounds:tt)*)),*))?,
        type In = $in:tt,
        |$id_self:ident| $id:expr,
        |$inner_ref_self:ident| $inner_ref:expr,
        |$inner_mut_self:ident| $inner_mut:expr,
        |$run_self:ident, $run_input:ident, $run_world:ident| $run:expr $(,)?
    ) => {
        impl $(<$($type_arg),*>)? System for $struct_name $(<$($type_arg),*>)?
        $(where $($($type_bounds)*),*)?
        {
            type In = $in;
            type Out = ShouldRun;

            fn name(&self) -> Cow<'static, str> {
                Cow::Borrowed(std::any::type_name::<Self>())
            }

            fn id($id_self: &Self) -> SystemId {
                $id
            }

            fn new_archetype($inner_mut_self: &mut Self, archetype: &Archetype) {
                let (_, archetype_component_access, criteria) = $inner_mut;
                for criterion in criteria {
                    criterion.new_archetype(archetype);
                    archetype_component_access.extend(criterion.archetype_component_access());
                }
            }

            fn component_access(&self) -> &Access<ComponentId> {
                &self.component_access
            }

            fn archetype_component_access(&self) -> &Access<ArchetypeComponentId> {
                &self.archetype_component_access
            }

            fn is_send($inner_ref_self: &Self) -> bool {
                let (_, _, criteria) = $inner_ref;
                criteria.fold(true, |acc, criterion| acc && criterion.is_send())
            }

            unsafe fn run_unsafe($run_self: &mut Self, $run_input: Self::In, $run_world: &World) -> Self::Out {
                $run
            }

            fn apply_buffers($inner_mut_self: &mut Self, world: &mut World) {
                let (_, _, criteria) = $inner_mut;
                for criterion in criteria {
                    criterion.apply_buffers(world);
                }
            }

            fn initialize($inner_mut_self: &mut Self, world: &mut World) {
                let (component_access, _, criteria) = $inner_mut;
                for criterion in criteria {
                    criterion.initialize(world);
                    component_access.extend(criterion.component_access());
                }
            }

            fn check_change_tick($inner_mut_self: &mut Self, change_tick: u32) {
                let (_, _, criteria) = $inner_mut;
                for criterion in criteria {
                    criterion.check_change_tick(change_tick);
                }
            }
        }
    };
}

macro_rules! impl_connected_run_criteria_0 {
    ($(#[$struct_attr:meta])* $struct_name:ident, $(#[$func_attr:meta])* $func_name:ident, |$self:ident, $input:ident, $world:ident| $run:block) => {
        $(#[$func_attr])*
        pub fn $func_name() -> impl System<In = ShouldRun, Out = ShouldRun> {
            $struct_name {
                connector: identity::<ShouldRun>.system(),
                system_id: SystemId::new(),
                archetype_component_access: Default::default(),
                component_access: Default::default(),
            }
        }

        $(#[$struct_attr])*
        struct $struct_name<C>
        where
            C: System<In = ShouldRun, Out = ShouldRun>,
        {
            connector: C,
            system_id: SystemId,
            archetype_component_access: Access<ArchetypeComponentId>,
            component_access: Access<ComponentId>,
        }

        impl_system_for_combinator!(
            $struct_name <C> (where (C: System<In = ShouldRun, Out = ShouldRun>)),
            type In = ShouldRun,
            |self| self.system_id,
            |self| {
                let Self {
                    ref component_access,
                    ref archetype_component_access,
                    ..
                } = self;
                (component_access, archetype_component_access, std::iter::empty::<&BoxedSystem<(), ()>>())
            },
            |self| {
                let Self {
                    ref mut component_access,
                    ref mut archetype_component_access,
                    ..
                } = self;
                (component_access, archetype_component_access, std::iter::empty::<&mut BoxedSystem<(), ()>>())
            },
            |$self, $input, $world| $run,
        );
    };
}

macro_rules! impl_connected_run_criteria_1 {
    ($(#[$struct_attr:meta])* $struct_name:ident, $(#[$func_attr:meta])* $func_name:ident, |$self:ident, $input:ident, $world:ident| $run:block) => {
        $(#[$func_attr])*
        pub fn $func_name(criterion: impl System<In = (), Out = ShouldRun>) -> impl System<In = ShouldRun, Out = ShouldRun> {
            $struct_name {
                connector: identity::<ShouldRun>.system(),
                criterion,
                system_id: SystemId::new(),
                archetype_component_access: Default::default(),
                component_access: Default::default(),
            }
        }

        $(#[$struct_attr])*
        struct $struct_name<C, S>
        where
            C: System<In = ShouldRun, Out = ShouldRun>,
            S: System<In = (), Out = ShouldRun>,
        {
            connector: C,
            criterion: S,
            system_id: SystemId,
            archetype_component_access: Access<ArchetypeComponentId>,
            component_access: Access<ComponentId>,
        }

        impl<C, S> System for $struct_name<C, S>
        where
            C: System<In = ShouldRun, Out = ShouldRun>,
            S: System<In = (), Out = ShouldRun>,
        {
            type In = ShouldRun;
            type Out = ShouldRun;

            fn name(&self) -> Cow<'static, str> {
                Cow::Borrowed(std::any::type_name::<Self>())
            }

            fn id(&self) -> SystemId {
                self.system_id
            }

            fn new_archetype(&mut self, archetype: &Archetype) {
                self.connector.new_archetype(archetype);
                self.criterion.new_archetype(archetype);

                self.archetype_component_access
                    .extend(self.connector.archetype_component_access());
                self.archetype_component_access
                    .extend(self.criterion.archetype_component_access());
            }

            fn component_access(&self) -> &Access<ComponentId> {
                &self.component_access
            }

            fn archetype_component_access(&self) -> &Access<ArchetypeComponentId> {
                &self.archetype_component_access
            }

            fn is_send(&self) -> bool {
                self.connector.is_send() && self.criterion.is_send()
            }

            unsafe fn run_unsafe($self: &mut Self, $input: Self::In, $world: &World) -> Self::Out {
                $run
            }

            fn apply_buffers(&mut self, world: &mut World) {
                self.connector.apply_buffers(world);
                self.criterion.apply_buffers(world);
            }

            fn initialize(&mut self, world: &mut World) {
                self.connector.initialize(world);
                self.criterion.initialize(world);

                self.component_access.extend(self.connector.component_access());
                self.component_access.extend(self.criterion.component_access());
            }

            fn check_change_tick(&mut self, change_tick: u32) {
                self.connector.check_change_tick(change_tick);
                self.criterion.check_change_tick(change_tick);
            }
        }
    };
}

macro_rules! impl_run_criteria_n {
    ($(#[$struct_attr:meta])* $struct_name:ident, $(#[$func_attr:meta])* $func_name:ident, |$self:ident, $input:ident, $world:ident| $run:block) => {
        $(#[$func_attr])*
        pub fn $func_name<In: Clone + 'static>(criteria: Vec<Box<dyn System<In = In, Out = ShouldRun>>>) -> impl System<In = In, Out = ShouldRun> {
            $struct_name {
                criteria,
                system_id: SystemId::new(),
                archetype_component_access: Default::default(),
                component_access: Default::default(),
            }
        }

        $(#[$struct_attr])*
        struct $struct_name<In>
        where
            In: Clone + 'static,
        {
            criteria: Vec<Box<dyn System<In = In, Out = ShouldRun>>>,
            system_id: SystemId,
            archetype_component_access: Access<ArchetypeComponentId>,
            component_access: Access<ComponentId>,
        }

        impl_system_for_combinator!(
            $struct_name <In> (where (In: Clone)),
            type In = In,
            |self| self.system_id,
            |self| {
                let Self {
                    ref criteria,
                    ref component_access,
                    ref archetype_component_access,
                    ..
                } = self;
                (component_access, archetype_component_access, criteria.iter())
            },
            |self| {
                let Self {
                    ref mut criteria,
                    ref mut component_access,
                    ref mut archetype_component_access,
                    ..
                } = self;
                (component_access, archetype_component_access, criteria.iter_mut())
            },
            |$self, $input, $world| $run,
        );
    };
}

fn split_should_run(should_run: ShouldRun) -> (bool, bool) {
    match should_run {
        ShouldRun::NoAndCheckAgain => (false, true),
        ShouldRun::No => (false, false),
        ShouldRun::YesAndCheckAgain => (true, true),
        ShouldRun::Yes => (true, false),
    }
}

fn merge_should_run(run: bool, check_again: bool) -> ShouldRun {
    match (run, check_again) {
        (false, false) => ShouldRun::No,
        (false, true) => ShouldRun::NoAndCheckAgain,
        (true, false) => ShouldRun::Yes,
        (true, true) => ShouldRun::YesAndCheckAgain,
    }
}

impl_connected_run_criteria_0!(
    NotIf,
    /// Intended to be used with [`RunCriteria::pipe()`] or
    /// [`IntoChainSystem::chain`]. Negates the result of the previous run
    /// criteria.
    ///
    /// Runs a system if:
    ///  - The preceeding criteria succeeds.
    ///
    /// Checks again if:
    ///  - The preceeding criteria requests to be checked again.
    not,
    |self, input, world| {
        match self.connector.run_unsafe(input, world) {
            ShouldRun::No => ShouldRun::Yes,
            ShouldRun::Yes => ShouldRun::No,
            ShouldRun::NoAndCheckAgain => ShouldRun::YesAndCheckAgain,
            ShouldRun::YesAndCheckAgain => ShouldRun::NoAndCheckAgain,
        }
    }
);

impl_connected_run_criteria_1!(
    AndIf,
    /// Intended to be used with [`RunCriteria::pipe()`] or
    /// [`IntoChainSystem::chain`]. Allows a run criterion that takes no
    /// [`ShouldRun`] as input to be piped/chained into.
    ///
    /// Runs a system if:
    ///  - Both the preceeding and inner criteria succeed.
    ///  - If the preceedding criteria fails, the inner criterion will not be
    ///    executed.
    ///
    /// Checks again if:
    ///  - The preceeding criteria returns [`ShouldRun::No`], then no.
    ///  - Otherwise, either the preceeding or inner criteria request to be
    ///    checked again.
    ///
    /// This is short-circuiting. If the preceeding run criteria returns
    /// [`ShouldRun::No`] or [`ShouldRun::NoAndCheckAgain`], then that result
    /// is returned and the inner criterion is not executed.
    and,
    |self, input, world| {
        let (connector_run, connector_check_again) =
            split_should_run(self.connector.run_unsafe(input, world));
        if connector_run {
            let (criterion_run, criterion_check_again) =
                split_should_run(self.criterion.run_unsafe((), world));
            merge_should_run(
                criterion_run,
                connector_check_again || criterion_check_again,
            )
        } else {
            merge_should_run(false, connector_check_again)
        }
    }
);

impl_connected_run_criteria_1!(
    OrIf,
    /// Intended to be used with [`RunCriteria::pipe()`] or
    /// [`IntoChainSystem::chain`]. Allows a run criterion that takes no
    /// [`ShouldRun`] as input to be piped/chained into.
    ///
    /// Runs a system if:
    ///  - Either the preceeding or inner criteria succeed.
    ///  - If the preceedding criteria succeeds, the inner criterion will not
    ///    be executed.
    ///
    /// Checks again if:
    ///  - The preceeding criteria returns [`ShouldRun::Yes`], then no.
    ///  - Otherwise, either the preceeding or inner criteria request to be
    ///    checked again.
    ///
    /// This is short-circuiting. If the preceeding run criteria returns
    /// [`ShouldRun::Yes`] or [`ShouldRun::YesAndCheckAgain`], then that result
    /// is returned and the inner criterion is not executed.
    or,
    |self, input, world| {
        let (connector_run, connector_check_again) =
            split_should_run(self.connector.run_unsafe(input, world));
        if connector_run {
            merge_should_run(true, connector_check_again)
        } else {
            let (criterion_run, criterion_check_again) =
                split_should_run(self.criterion.run_unsafe((), world));
            merge_should_run(
                criterion_run,
                connector_check_again || criterion_check_again,
            )
        }
    }
);

impl_run_criteria_n!(
    RunIfAll,
    /// Runs a system if:
    ///  - All the inner criteria succeed (or if there are no inner criteria).
    ///  - If any criterion fails, then any subsequent criteria will not be
    ///    executed.
    ///
    /// Checks again if:
    ///  - Any of the executed inner criteria request to be checked again.
    ///
    /// This is short-circuiting. If any inner criteria return
    /// [`ShouldRun::No`] or [`ShouldRun::NoAndCheckAgain`], then that result
    /// is returned and the rest are not executed.
    ///
    /// Criteria execution order follows the order in the inner vector.
    if_all,
    |self, input, world| {
        let mut check_again = false;
        let run = self.criteria.iter_mut().all(|criterion| {
            let (run, cur_check_again) =
                split_should_run(criterion.run_unsafe(input.clone(), world));
            check_again |= cur_check_again;
            run
        });

        merge_should_run(run, check_again)
    }
);

impl_run_criteria_n!(
    RunIfAny,
    /// Runs a system if:
    ///  - Any the inner criteria succeed (and if there are inner criteria).
    ///  - If any criterion succeeds, then any subsequent criteria will not be
    ///    executed.
    ///
    /// Checks again if:
    ///  - Any of the executed inner criteria request to be checked again.
    ///
    /// This is short-circuiting. If any inner criteria return
    /// [`ShouldRun::Yes`] or [`ShouldRun::YesAndCheckAgain`], then that result
    /// is returned and the rest are not executed.
    ///
    /// Criteria execution order follows the order in the inner vector.
    if_any,
    |self, input, world| {
        let mut check_again = false;
        let run = self.criteria.iter_mut().any(|criterion| {
            let (run, cur_check_again) =
                split_should_run(criterion.run_unsafe(input.clone(), world));
            check_again |= cur_check_again;
            run
        });

        merge_should_run(run, check_again)
    }
);
