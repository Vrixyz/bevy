//! Demonstrates handling a key press/release.

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Update, keyboard_input_system)
        .run();
}

/// This system prints 'A' key state
fn keyboard_input_system(keyboard_input: Res<Input<Key>>) {
    if keyboard_input.pressed("a") {
        info!("'A' currently pressed");
    }

    if keyboard_input.just_pressed("a") {
        info!("'A' just pressed");
    }

    if keyboard_input.just_released("a") {
        info!("'A' just released");
    }
    if keyboard_input.pressed("A") {
        info!("capital 'A' is being pressed");
    }
}
