mod shared;

use std::sync::Arc;
use bevy::prelude::*;
use bevy_stardust_quinn::RootCertStore;

fn root_certs() -> Arc<RootCertStore> {
    let mut certs = RootCertStore::empty();
    certs.add(shared::certificate()).unwrap();
    return Arc::new(certs);
}

fn main() {
    let mut app = App::new();

    shared::setup(&mut app);

    app.run();
}