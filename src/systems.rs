//! System for the navigation tree and default input systems to get started
use crate::events::{Direction, NavRequest, ScopeDirection};
use crate::{max_by_in_iter, Focusable, Focused};
use bevy::ecs::system::SystemParam;
use bevy::math::Vec3Swizzles;
use bevy::prelude::*;
use bevy::ui::camera::CAMERA_UI;
use bevy::render::camera::Camera;

/// Control default ui navigation input buttons
pub struct InputMapping {
    /// Deadzone on the gamepad left stick for ui navigation
    pub joystick_ui_deadzone: f32,
    /// X axis of gamepad stick
    pub move_x: GamepadAxisType,
    /// Y axis of gamepad stick
    pub move_y: GamepadAxisType,
    /// X axis of gamepad dpad
    pub move_x_dpad: GamepadAxisType,
    /// Y axis of gamepad dpad
    pub move_y_dpad: GamepadAxisType,
    /// Gamepad button for [`NavRequest::Action`]
    pub action_button: GamepadButtonType,
    /// Gamepad button for [`NavRequest::Cancel`]
    pub cancel_button: GamepadButtonType,
    /// Gamepad button for [`ScopeDirection::Previous`] [`NavRequest::ScopeMove`]
    pub previous_button: GamepadButtonType,
    /// Gamepad button for [`ScopeDirection::Next`] [`NavRequest::ScopeMove`]
    pub next_button: GamepadButtonType,
    /// Keyboard key for [`Direction::West`] [`NavRequest::Move`]
    pub key_left: KeyCode,
    /// Keyboard key for [`Direction::East`] [`NavRequest::Move`]
    pub key_right: KeyCode,
    /// Keyboard key for [`Direction::North`] [`NavRequest::Move`]
    pub key_up: KeyCode,
    /// Keyboard key for [`Direction::South`] [`NavRequest::Move`]
    pub key_down: KeyCode,
    /// Alternative keyboard key for [`Direction::West`] [`NavRequest::Move`]
    pub key_left_alt: KeyCode,
    /// Alternative keyboard key for [`Direction::East`] [`NavRequest::Move`]
    pub key_right_alt: KeyCode,
    /// Alternative keyboard key for [`Direction::North`] [`NavRequest::Move`]
    pub key_up_alt: KeyCode,
    /// Alternative keyboard key for [`Direction::South`] [`NavRequest::Move`]
    pub key_down_alt: KeyCode,
    /// Keyboard key for [`NavRequest::Action`]
    pub key_action: KeyCode,
    /// Keyboard key for [`NavRequest::Cancel`]
    pub key_cancel: KeyCode,
    /// Keyboard key for [`ScopeDirection::Next`] [`NavRequest::ScopeMove`]
    pub key_next: KeyCode,
    /// Alternative keyboard key for [`ScopeDirection::Next`] [`NavRequest::ScopeMove`]
    pub key_next_alt: KeyCode,
    /// Keyboard key for [`ScopeDirection::Previous`] [`NavRequest::ScopeMove`]
    pub key_previous: KeyCode,
    /// Mouse button for [`NavRequest::Action`]
    pub mouse_action: MouseButton,
}
impl Default for InputMapping {
    fn default() -> Self {
        InputMapping {
            joystick_ui_deadzone: 0.36,
            move_x: GamepadAxisType::LeftStickX,
            move_y: GamepadAxisType::LeftStickY,
            move_x_dpad: GamepadAxisType::DPadX,
            move_y_dpad: GamepadAxisType::DPadY,
            action_button: GamepadButtonType::South,
            cancel_button: GamepadButtonType::East,
            previous_button: GamepadButtonType::LeftTrigger,
            next_button: GamepadButtonType::RightTrigger,
            key_left: KeyCode::A,
            key_right: KeyCode::D,
            key_up: KeyCode::W,
            key_down: KeyCode::S,
            key_left_alt: KeyCode::Left,
            key_right_alt: KeyCode::Right,
            key_up_alt: KeyCode::Up,
            key_down_alt: KeyCode::Down,
            key_action: KeyCode::Space,
            key_cancel: KeyCode::Back,
            key_next: KeyCode::E,
            key_next_alt: KeyCode::Tab,
            key_previous: KeyCode::Q,
            mouse_action: MouseButton::Left,
        }
    }
}

/// `mapping { XYZ::X => ABC::A, XYZ::Y => ABC::B, XYZ::Z => ABC::C }: [(XYZ, ABC)]`
macro_rules! mapping {
    ($($from:expr => $to:expr),* ) => ([$( ( $from, $to ) ),*])
}

/// A system to send gamepad control events to the focus system
///
/// Dpad and left stick for movement, `LT` and `RT` for scopped menus, `A` `B`
/// for selection and cancel.
///
/// The button mapping may be controlled through the [`InputMapping`] resource.
/// You may however need to customize the behavior of this system (typically
/// when integrating in the game) in this case, you should write your own
/// system that sends [`NavRequest`](crate::NavRequest) events
pub fn default_gamepad_input(
    mut nav_cmds: EventWriter<NavRequest>,
    input_mapping: Res<InputMapping>,
    buttons: Res<Input<GamepadButton>>,
    axis: Res<Axis<GamepadAxis>>,
    mut ui_input_status: Local<bool>,
) {
    use Direction::*;
    use NavRequest::{Action, Cancel, Move, ScopeMove};

    let pad = Gamepad(0);
    macro_rules! axis_delta {
        ($dir:ident, $axis:ident) => {
            axis.get(GamepadAxis(pad, input_mapping.$axis))
                .map_or(Vec2::ZERO, |v| Vec2::$dir * v)
        };
    }

    let stick_move = axis_delta!(Y, move_y) + axis_delta!(X, move_x);
    let dpad_move = axis_delta!(Y, move_y_dpad) + axis_delta!(X, move_x_dpad);
    let dpad_greater = dpad_move.length_squared() > stick_move.length_squared();
    let delta = if dpad_greater { dpad_move } else { stick_move };
    if delta.length_squared() > input_mapping.joystick_ui_deadzone && !*ui_input_status {
        let direction = match () {
            () if delta.y < delta.x && delta.y < -delta.x => South,
            () if delta.y > delta.x && delta.y > -delta.x => North,
            () if delta.y < delta.x && delta.y > -delta.x => East,
            () if delta.y > delta.x && delta.y < -delta.x => West,
            () => unreachable!(),
        };
        nav_cmds.send(Move(direction));
        *ui_input_status = true;
    } else if delta.length_squared() <= input_mapping.joystick_ui_deadzone {
        *ui_input_status = false;
    }

    let command_mapping = mapping! {
        input_mapping.action_button => Action,
        input_mapping.cancel_button => Cancel,
        input_mapping.next_button => ScopeMove(ScopeDirection::Next),
        input_mapping.previous_button => ScopeMove(ScopeDirection::Previous)
    };
    for (key, request) in command_mapping {
        if buttons.just_pressed(GamepadButton(pad, key)) {
            nav_cmds.send(request)
        }
    }
}

/// A system to send keyboard control events to the focus system
///
/// supports `WASD` and arrow keys for the directions, `E`, `Q` and `Tab` for
/// scopped menus, `Backspace` and `Enter` for cancel and selection
///
/// The button mapping may be controlled through the [`InputMapping`] resource.
/// You may however need to customize the behavior of this system (typically
/// when integrating in the game) in this case, you should write your own
/// system that sends [`NavRequest`](crate::NavRequest) events
pub fn default_keyboard_input(
    keyboard: Res<Input<KeyCode>>,
    input_mapping: Res<InputMapping>,
    mut nav_cmds: EventWriter<NavRequest>,
) {
    use Direction::*;
    use NavRequest::*;

    let command_mapping = mapping! {
        input_mapping.key_action => Action,
        input_mapping.key_cancel => Cancel,
        input_mapping.key_up => Move(North),
        input_mapping.key_down => Move(South),
        input_mapping.key_left => Move(West),
        input_mapping.key_right => Move(East),
        input_mapping.key_up_alt => Move(North),
        input_mapping.key_down_alt => Move(South),
        input_mapping.key_left_alt => Move(West),
        input_mapping.key_right_alt => Move(East),
        input_mapping.key_next => ScopeMove(ScopeDirection::Next),
        input_mapping.key_next_alt => ScopeMove(ScopeDirection::Next),
        input_mapping.key_previous => ScopeMove(ScopeDirection::Previous)
    };
    for (key, request) in command_mapping {
        if keyboard.just_pressed(key) {
            nav_cmds.send(request)
        }
    }
}

#[derive(SystemParam)]
pub struct NodePosQuery<'w, 's> {
    entities: Query<'w, 's, (Entity, &'static Node, &'static GlobalTransform), With<Focusable>>,
    cam_names: Query<'w, 's, (&'static Camera, &'static GlobalTransform)>,
}

fn is_in_node(at: Vec2, (_, node, trans): &(Entity, &Node, &GlobalTransform)) -> bool {
    let ui_pos = trans.translation.truncate();
    let node_half_size = node.size / 2.0;
    let min = ui_pos - node_half_size;
    let max = ui_pos + node_half_size;
    (min.x..max.x).contains(&at.x) && (min.y..max.y).contains(&at.y)
}

/// Check which [`Focusable`] displays below `at` if any
pub fn ui_focusable_at(at: Vec2, query: &NodePosQuery) -> Option<Entity> {
    let ui_cam_name = query
        .cam_names
        .iter()
        .find(|cam| cam.0.name.as_deref() == Some(CAMERA_UI));
    let ui_camera_position = if let Some((_, trans)) = ui_cam_name {
        trans.translation.xy()
    } else {
        Vec2::ZERO
    };
    let under_mouse = query
        .entities
        .iter()
        .filter(|query_elem| is_in_node(at + ui_camera_position, query_elem));
    max_by_in_iter(under_mouse, |elem| elem.2.translation.z).map(|elem| elem.0)
}

fn cursor_pos(windows: &Windows) -> Option<Vec2> {
    windows.get_primary().and_then(|w| w.cursor_position())
}

/// A system to send mouse control events to the focus system
///
/// Which button to press to cause an action event is specified in the
/// [`InputMapping`] resource.
///
/// You may however need to customize the behavior of this system (typically
/// when integrating in the game) in this case, you should write your own
/// system that sends [`NavRequest`](crate::NavRequest) events. You may use
/// [`ui_focusable_at`] to tell which focusable is currently being hovered.
#[allow(clippy::too_many_arguments)]
pub fn default_mouse_input(
    input_mapping: Res<InputMapping>,
    windows: Res<Windows>,
    mouse: Res<Input<MouseButton>>,
    touch: Res<Touches>,
    focusables: NodePosQuery,
    focused: Query<Entity, With<Focused>>,
    mut nav_cmds: EventWriter<NavRequest>,
    mut last_pos: Local<Vec2>,
) {
    let ui_cam_name = focusables
        .cam_names
        .iter()
        .find(|cam| cam.0.name.as_deref() == Some(CAMERA_UI));
    let ui_camera_position = if let Some((_, trans)) = ui_cam_name {
        trans.translation.xy()
    } else {
        Vec2::ZERO
    };
    let cursor_pos = match cursor_pos(&windows) {
        Some(c) => c,
        None => return,
    };
    let released = mouse.just_released(input_mapping.mouse_action) || touch.just_released(0);
    let focused = focused.get_single();
    // Return early if cursor didn't move since last call
    if !released && *last_pos == cursor_pos {
        return;
    } else {
        *last_pos = cursor_pos;
    }
    let not_hovering_focused = |focused: &Entity| {
        let focused = focusables
            .entities
            .get(*focused)
            .expect("Entity with `Focused` component must also have a `Focusable` component");
        !is_in_node(cursor_pos + ui_camera_position, &focused)
    };
    // If the currently hovered node is the focused one, there is no need to
    // find which node we are hovering and to switch focus to it (since we are
    // already focused on it)
    if focused.iter().all(not_hovering_focused) {
        // `ui_focusable_at` is computationally heavy
        let to_target = match ui_focusable_at(cursor_pos, &focusables) {
            Some(c) => c,
            None => return,
        };
        nav_cmds.send(NavRequest::FocusOn(to_target));
    } else if released {
        nav_cmds.send(NavRequest::Action);
    }
}
