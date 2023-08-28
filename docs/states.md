# MultiplayerState state machine
```mermaid
flowchart TD
    Disconnected
        Disconnected-->Singleplayer
        Disconnected-->StartingServer
        Disconnected-->JoiningRemote
    Singleplayer
        Singleplayer-->Disconnected
        Singleplayer-->StartingServer
        Singleplayer-->JoiningRemote
    StartingServer
        StartingServer-->RunningServer
        StartingServer-->Disconnected
    RunningServer
        RunningServer-->Disconnected
    JoiningRemote
        JoiningRemote-->Disconnected
        JoiningRemote-->JoinedRemote
    JoinedRemote
        JoinedRemote-->Disconnected
```

The App will always start at `Disconnected`, *unless* the `MultiplayerMode` is `DedicatedServer`, where it will start at `StartingServer`.

`Singleplayer` can become `StartingServer` or `JoiningRemote` without being `Disconnected` first. This is because 'sudden multiplayer' exists, ie Dark Souls and Watch Dogs 2's invasions. Transport layers should be prepared for this sudden state change, but the specifics of how it works is up to the game developer.

Some states are **prohibited** based on the `MultiplayerMode`, ie. a `DedicatedServer` cannot be in state `JoiningRemote`. If this occurs, the app will panic.