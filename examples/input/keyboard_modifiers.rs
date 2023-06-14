//! Demonstrates using key modifiers (ctrl, shift).

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Update, keyboard_input_system)
        .run();
}

/// This system prints when `Ctrl + Shift + A` is pressed
fn keyboard_input_system(input: Res<Input<Key>>) {
    let shift = input.pressed(Key::Shift);
    let ctrl = input.pressed(Key::Control);

    if ctrl && shift && input.just_pressed(Key::Character("a".into())) {
        info!("Just pressed Ctrl + Shift + A!");
    }
}
