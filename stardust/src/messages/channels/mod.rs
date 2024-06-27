//! Message organisation systems.
//! 
//! # Defining a channel
//! Channels are accessed using the type system. You can use any type,
//! as long as it implements [`Channel`]. Since `Channel` is automatically
//! implemented for any type that is `TypePath + Send + Sync + 'static`,
//! in almost all cases you just have to derive `TypePath`.
//! 
//! ```no_run
//! # use bevy::prelude::*;
//! # use bevy_stardust::prelude::*;
//! # use std::marker::PhantomData;
//! #
//! // Defining a channel type is simple.
//! #[derive(TypePath)]
//! pub struct MyChannel;
//! 
//! // You can make channels follow Rust's privacy system, checked at compile time.
//! #[derive(TypePath)]
//! struct MyPrivateChannel;
//! 
//! // You can make channels with generic type bounds too, as long as the resulting type is still TypePath.
//! #[derive(TypePath)]
//! struct MyGenericChannel<T: Channel>(PhantomData<T>);
//! 
//! // Channels don't have to be zero-sized types.
//! // They are never instantiated anywhere, and only compile-time data is ever used.
//! #[derive(TypePath)]
//! enum MySizedChannel {
//!     Foo,
//!     Bar,
//!     Qux(String),
//! }
//! 
//! // If the channel contains serialised messages, you can use that type as the channel.
//! // This is useful for avoiding cluttered networking code, and forming better mental models.
//! #[derive(TypePath)]
//! struct MovementEvent {
//!     direction: Vec2,
//! }
//! ```
//! 
//! # Adding a channel
//! Channels must be added to the `App` before being used.
//! To do this, just use the [`add_channel<C>`][add_channel] on the `App`.
//! This is implemented by the [`ChannelSetupAppExt`] trait, which is
//! automatically brought into scope with `use bevy_stardust::prelude::*;`
//! 
//! [`add_channel`][add_channel] takes a generic, `C`, which you should
//! set as the type of the channel you are trying to add.
//! 
//! Note that channels must be added *after* [`StardustPlugin`] is added,
//! and *before* `StardustPlugin` [finishes][Plugin::finish]. Channel
//! insertion order also matters. You must make sure all calls to
//! [`add_channel`][add_channel] are in a deterministic order.
//! 
//! [`StardustPlugin`]: crate::plugin::StardustPlugin
//! [Plugin::finish]: bevy::prelude::Plugin::finish
//! [add_channel]: ChannelSetupAppExt::add_channel
//! 
//! ```no_run
//! # use bevy::prelude::*;
//! # use bevy_stardust::prelude::*;
//! 
//! #[derive(TypePath, Event)]
//! pub struct MovementEvent(pub Vec3);
//!
//! fn main() {
//!     let mut app = App::new();
//! 
//!     app.add_plugins((DefaultPlugins, StardustPlugin));
//! 
//!     app.add_event::<MovementEvent>();
//!     app.add_channel::<MovementEvent>(ChannelConfiguration {
//!         consistency: ChannelConsistency::UnreliableUnordered,
//!         priority: 0,
//!     });
//! 
//!     app.run();
//! }
//! ```

mod config;
mod id;
mod registry;
mod extension;

pub use config::*;
pub use id::*;
pub use registry::*;
pub use extension::*;