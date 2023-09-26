//! Demonstrates handling a key press/release.

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Update, keyboard_input_system)
        .run();
}

/// This system prints 'A' character key state
fn keyboard_input_system(keyboard_input: Res<Input<KeyLogic>>) {
    if keyboard_input.pressed("a") {
        info!("Character lowercase 'a' currently pressed");
    }

    if keyboard_input.just_pressed("a") {
        info!("Character lowercase 'a' just pressed");
    }

    if keyboard_input.just_released("a") {
        info!("Character lowercase 'a' just released");
    }

    if keyboard_input.pressed("A") {
        info!("Character uppercase 'A' currently pressed");
    }
    if keyboard_input.just_pressed("A") {
        info!("Character uppercase 'A' just pressed");
    }
    if keyboard_input.just_released("A") {
        info!("Character uppercase 'A' just released");
    }
}
