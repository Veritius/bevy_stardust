mod shared;

use bevy::prelude::*;

fn main() {
    let mut app = App::new();

    shared::setup(&mut app);

    app.run();
}