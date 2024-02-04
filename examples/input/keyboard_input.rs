//! Demonstrates handling a key press/release.

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Update, keyboard_input_system)
        .run();
}

/// This system prints 'A' key state
fn keyboard_input_system(keyboard_input: Res<ButtonInput<PhysicalKey>>) {
    if keyboard_input.pressed(PhysicalKey::KeyA) {
        info!("'A' currently pressed");
    }

    if keyboard_input.just_pressed(PhysicalKey::KeyA) {
        info!("'A' just pressed");
    }
    if keyboard_input.just_released(PhysicalKey::KeyA) {
        info!("'A' just released");
    }
}
