//! 'Modifiers' that can be attached to replicated components and resources to change their behavior.

use std::marker::PhantomData;
use aery::prelude::*;
use bevy::prelude::*;

macro_rules! override_relation {
    ($name:ident, $doc:expr) => {
        /// A relation that overrides behavior for a specific peer or group to
        #[doc=$doc]
        /// regardless of default configuration.
        /// 
        /// The `T` generic controls what part of an entity is targeted.
        /// By default, `T` is `Entity`, which affects the entire entity.
        /// `T` can be changed to the type of any `Component` to target it.
        #[derive(Relation)]
        pub struct $name<T = Entity>(PhantomData<T>);
    };
}

override_relation!(Visible, "always show the target");
override_relation!(Hidden, "always hide the target");
override_relation!(Thawed, "always update the target");
override_relation!(Frozen, "never update the target");