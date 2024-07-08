# bevy_stardust_quic
A QUIC-based transport layer for bevy_stardust.

## Backends
This plugin does not use a specific QUIC implementation.
Instead, you can choose from the currently supported implementations.

| Name   | Feature flag | License      | Repository                           |
|--------|--------------|--------------|--------------------------------------|
| quiche | `quiche`     | BSD 2-Clause | https://github.com/cloudflare/quiche |