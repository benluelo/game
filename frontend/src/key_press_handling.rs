use bevy::{
    ecs::schedule::SystemDescriptor,
    prelude::*,
    utils::{StableHashMap, StableHashSet},
};

use num_traits::Zero;

pub fn system() -> impl Into<SystemDescriptor> {
    key_press_handling.system().label(KeyPressHandling)
}

/// [`SystemLabel`] for [`key_press_handling`].
#[derive(SystemLabel, Debug, PartialEq, Eq, Clone, Hash)]
pub struct KeyPressHandling;

/// Contains all the keys that are currently pressed and how long they have been
/// pressed.
#[derive(Debug)]
pub struct KeyPressTime(pub StableHashMap<KeyCode, f32>);

// ANCHOR[id=key_press_handling]
/// Collects all the keys that are pressed and maps them to how long they have
/// been pressed.
pub fn key_press_handling(
    mut key_press_time: ResMut<KeyPressTime>,
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let pressed = keyboard_input.get_pressed().collect::<StableHashSet<_>>();

    let newly_pressed_keys = keyboard_input
        .get_pressed()
        .filter(|x| !key_press_time.0.contains_key(*x))
        .map(|x| (*x, f32::zero()));

    key_press_time.0 = key_press_time
        .0
        .iter()
        .filter_map(|(k, v)| {
            if pressed.contains(k) {
                Some((*k, *v + time.delta_seconds()))
            } else {
                None
            }
        })
        .chain(newly_pressed_keys)
        .collect();
    // dbg!(&*key_press_time);
}
