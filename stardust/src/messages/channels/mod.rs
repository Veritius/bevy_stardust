//! Message organisation systems.
//! 
//! # What are channels
//! Channels are an abstraction provided by Stardust to make writing netcode easier.
//! Instead of having a static number like `12345` to identify message types,
//! Stardust automatically generates these numbers, which have the dual benefit of
//! being very efficient to transmit, and easy to work with for a developer.
//! Most of the time, you as the developer won't directly work with channel identifiers.
//! Instead, you use the type system, just like you would to use Bevy systemparams.
//! 
//! A major benefit of automatically generating channel identifiers is that
//! it's incredibly easy to add new message types. You don't need a massive
//! document of every channel ID to make sure that system A doesn't read a
//! message intended for system B. This is especially useful when using plugins,
//! which now just work, with no extra effort on your part.
//! 
//! Channels also obey Rust's visibility system. Since you primarily access
//! channels with their associated type, if that type is not accessible,
//! that channel cannot be accessed, letting you compartmentalise code better.
//! This aligns very well with the compartmentalisation that ECS is designed for.
//! 
//! Note that you *can* technically access a channel without a type, using its ID,
//! but this is very unreliable and considered bad practice. Visibility cannot be
//! perfectly enforced, as transport layers must have access to all channels to
//! be able to do their job.
//! 
//! # Adding channels
//! Channels are accessed using the type system. You can use any type,
//! as long as it implements [`Channel`]. Since `Channel` is automatically
//! implemented for any type that is `TypePath + Send + Sync + 'static`,
//! in almost all cases you just have to derive `TypePath`.
//! 
//! ```no_run
//! # use bevy::prelude::*;
//! # use bevy_stardust::prelude::*;
//! #
//! #[derive(TypePath)]
//! pub struct MyChannel;
//! ```
//! 
//! Channels must also have a [`ChannelConfiguration`].
//! The configuration of a channel is used to tell transport layers how to treat
//! [messages] sent over that channel, like if messages should be ordered.
//! It's highly recommended to read the documentation of `ChannelConfiguration`
//! to understand what each field does, and its implications for your code.
//! 
//! ```no_run
//! # use bevy::prelude::*;
//! # use bevy_stardust::prelude::*;
//! # fn _p() {
//! let config = ChannelConfiguration {
//!     consistency: ChannelConsistency::ReliableOrdered,
//!     priority: 128,
//! };
//! # }
//! ```
//! 
//! Channels must be added to the `App` before being used. This is done
//! by adding it to the channel registry. To do this, just use the
//! [`add_channel<C>`][add_channel] on the `App`. This is implemented by
//! the [`ChannelSetupAppExt`] trait, which is automatically brought into
//! scope with `use bevy_stardust::prelude::*;`
//! 
//! [`add_channel`][add_channel] takes a generic, `C`, which you should
//! set as the type of the channel you are trying to add. In this case, our
//! channel is named `MyChannel`, so we would do `add_channel<MyChannel>`.
//! This function also takes the configuration of the channel. This is where
//! you put in the `ChannelConfiguration` you defined.
//! 
//! ```no_run
//! # use bevy::prelude::*;
//! # use bevy_stardust::prelude::*;
//! #
//! #[derive(TypePath)]
//! pub struct MyChannel;
//! 
//! fn main() {
//!     // Normal Bevy app setup
//!     let mut app = App::new();
//!     app.add_plugins(DefaultPlugins);
//! 
//!     // StardustPlugin must be added
//!     app.add_plugins(StardustPlugin);
//! 
//!     app.add_channel::<MyChannel>(ChannelConfiguration {
//!         consistency: ChannelConsistency::ReliableOrdered,
//!         priority: 128,
//!     });
//! }
//! ```
//! 
//! Note that channels must be added *after* [`StardustPlugin`] is added,
//! and *before* `StardustPlugin` [finishes][Plugin::finish]. Channel
//! insertion order also matters: you must make sure all calls to
//! [`add_channel`][add_channel] are in a deterministic order.
//! This includes channels registered by plugins.
//! This is an unfortunate limitation that will (hopefully) be lifted in future.
//! 
//! [messages]: crate::messages
//! [`StardustPlugin`]: crate::plugin::StardustPlugin
//! [Plugin::finish]: bevy::prelude::Plugin::finish
//! [add_channel]: ChannelSetupAppExt::add_channel
//! 
//! # Advanced channels
//! Only compile-time information is used from channel types.
//! Your types will never be instantiated. While most often,
//! you will use ZSTs like unit-like structs or zero-variant enums,
//! any type that implements `Channel` can be used.
//! 
//! Let's say you have an event called `MovementEvent`, that is
//! used to inform systems of player movements. If you want to
//! send this event to other peers, you could create a new type
//! and use it in `add_channel`, or you could use `MovementEvent`.
//! 
//! ```no_run
//! # use bevy::prelude::*;
//! # use bevy_stardust::prelude::*;
//! #
//! #[derive(Event, TypePath)]
//! struct MovementEvent {
//!     change: Vec3,
//! }
//! 
//! fn main() {
//!     // App setup as detailed earlier
//!     let mut app = App::new();
//!     app.add_plugins((DefaultPlugins, StardustPlugin));
//! 
//!     // Register MovementEvent as an event
//!     app.add_event::<MovementEvent>();
//! 
//!     // And register it as a channel
//!     app.add_channel::<MovementEvent>(ChannelConfiguration {
//!         consistency: ChannelConsistency::UnreliableUnordered,
//!         priority: 32,
//!     });
//! }
//! ```
//! 
//! At this point, you can introduce a system in `NetworkRecv::Read`
//! to turn the messages on your `MovementEvent` channel into events
//! in `Events<MovementEvent>`, which other systems can read from.
//! This can be useful to make your code less cluttered, especially
//! for replicated events like this, but there are times where it's
//! not suitable. It's up to you to decide when you want to use this.
//! 
//! You can also use generic type parameters as an organisational tool.
//! As long as the type still implements `Channel`, it's just fine!
//! ```no_run
//! # use bevy::prelude::*;
//! # use bevy_stardust::prelude::*;
//! # use std::marker::PhantomData;
//! #
//! #[derive(TypePath)]
//! pub struct MyGenericChannel<C: Channel>(PhantomData<C>);
//! ```

mod config;
mod id;
mod registry;
mod extension;

pub use config::*;
pub use id::*;
pub use registry::*;
pub use extension::*;

use bevy::prelude::*;

pub(crate) fn build_channels(app: &mut App) {
    app.insert_resource(ChannelRegistryBuilder(ChannelRegistry::new()));
}

pub(crate) fn finish_channels(app: &mut App) {
    let builder = app.world.remove_resource::<ChannelRegistryBuilder>().unwrap();
    app.world.insert_resource(builder.finish());
}