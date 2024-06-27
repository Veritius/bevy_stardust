//! Message organisation systems.
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
//! insertion order also matters. You must make sure all calls to
//! [`add_channel`][add_channel] are in a deterministic order.
//! This is an unfortunate limitation that will be lifted in future.
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