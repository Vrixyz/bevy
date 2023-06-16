//! Demonstrates handling a key press/release.

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Update, keyboard_input_system)
        .run();
}

/// This system prints 'A' key state
fn keyboard_input_system(keyboard_input: Res<Input<KeyLogic>>) {
    if keyboard_input.pressed("a") {
        info!("'a' currently pressed");
    }

    if keyboard_input.just_pressed("a") {
        info!("'a' just pressed");
    }

    if keyboard_input.just_released("a") {
        info!("'a' just released");
    }

    if keyboard_input.pressed("A") {
        info!("capital 'A' currently pressed");
    }
    if keyboard_input.just_pressed("A") {
        info!("capital 'A' just pressed");
    }
    if keyboard_input.just_released("A") {
        info!("capital 'A' just released");
    }
}
