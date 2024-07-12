# bevy_stardust_quic
A QUIC-based transport layer for bevy_stardust.

| Stardust | Crate   |
|----------|---------|
| `0.6.0`  | `0.1.0` |

## Backends
This plugin does not use a specific QUIC implementation.
Instead, you can choose from the currently supported implementations.

| Name   | Feature flag | License      | Repository                             | Additional build requirements |
|--------|--------------|--------------|----------------------------------------|-------------------------------|
| quiche | `quiche`     | BSD 2-Clause | <https://github.com/cloudflare/quiche> | [Extensive][quiche_building]  |

You can also enable the `reveal` feature flag to expose various backend-specific functionality.
This is not recommended for almost all applications, but there are some niche cases where it's needed.

[quiche_building]: https://github.com/cloudflare/quiche/tree/master?tab=readme-ov-file#building