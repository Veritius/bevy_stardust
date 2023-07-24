# Using Stardust


## Adding to Stardust with Bevy plugins
Using the `expose_internals` feature flag lets you view some of the nuts and bolts of Stardust, useful if you're looking to optimise your network code.

**Put anything and everything that changes the protocol in a shared crate, or another solution that ensures that the protocol is identical on both the client and server.**