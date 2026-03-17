#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CopyAction {
    Image,
    Path,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct CopyShortcutState {
    pub(super) wants_keyboard_input: bool,
    pub(super) has_copy_event: bool,
    pub(super) key_copy_path: bool,
    pub(super) key_copy_image: bool,
    pub(super) active_shift: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct CopyDecision {
    pub(super) action: Option<CopyAction>,
    pub(super) clear_hint: bool,
    pub(super) consume_copy_event: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct CopyShortcutSignals {
    pub(super) has_copy_event: bool,
    pub(super) key_copy_path: bool,
    pub(super) key_copy_image: bool,
    pub(super) active_shift: bool,
}

fn is_copy_modifier(modifiers: &egui::Modifiers) -> bool {
    #[cfg(target_os = "macos")]
    {
        modifiers.mac_cmd
    }
    #[cfg(not(target_os = "macos"))]
    {
        modifiers.ctrl
    }
}

pub(super) fn collect_copy_shortcut_signals(
    events: &[egui::Event],
    active_shift: bool,
) -> CopyShortcutSignals {
    let mut key_copy_path = false;
    let mut key_copy_image = false;

    for event in events {
        if let egui::Event::Key {
            key,
            pressed,
            modifiers,
            ..
        } = event
        {
            if !pressed || *key != egui::Key::C {
                continue;
            }
            if !is_copy_modifier(modifiers) {
                continue;
            }
            if modifiers.shift {
                key_copy_path = true;
            } else {
                key_copy_image = true;
            }
        }
    }

    let has_copy_event = events
        .iter()
        .any(|event| matches!(event, egui::Event::Copy));

    CopyShortcutSignals {
        has_copy_event,
        key_copy_path,
        key_copy_image,
        active_shift,
    }
}

pub(super) fn resolve_copy_action(state: CopyShortcutState) -> CopyDecision {
    if state.wants_keyboard_input {
        if state.key_copy_path || (state.has_copy_event && state.active_shift) {
            return CopyDecision {
                action: None,
                clear_hint: true,
                consume_copy_event: state.has_copy_event && state.active_shift,
            };
        }
        if state.key_copy_image || state.has_copy_event {
            return CopyDecision {
                action: None,
                clear_hint: true,
                consume_copy_event: false,
            };
        }
        return CopyDecision {
            action: None,
            clear_hint: false,
            consume_copy_event: false,
        };
    }

    let copy_path = state.key_copy_path || (state.has_copy_event && state.active_shift);
    if copy_path {
        return CopyDecision {
            action: Some(CopyAction::Path),
            clear_hint: false,
            consume_copy_event: false,
        };
    }

    let copy_image = state.key_copy_image || (state.has_copy_event && !state.active_shift);
    if copy_image {
        return CopyDecision {
            action: Some(CopyAction::Image),
            clear_hint: false,
            consume_copy_event: false,
        };
    }

    CopyDecision {
        action: None,
        clear_hint: false,
        consume_copy_event: false,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        collect_copy_shortcut_signals, resolve_copy_action, CopyAction, CopyShortcutState,
    };

    fn copy_modifiers(shift: bool) -> egui::Modifiers {
        let mut modifiers = egui::Modifiers::default();
        modifiers.shift = shift;
        #[cfg(target_os = "macos")]
        {
            modifiers.mac_cmd = true;
        }
        #[cfg(not(target_os = "macos"))]
        {
            modifiers.ctrl = true;
        }
        modifiers
    }

    #[test]
    fn integration_cmd_or_ctrl_c_event_to_copy_image_action() {
        let events = vec![egui::Event::Key {
            key: egui::Key::C,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: copy_modifiers(false),
        }];
        let signals = collect_copy_shortcut_signals(&events, false);
        let decision = resolve_copy_action(CopyShortcutState {
            wants_keyboard_input: false,
            has_copy_event: signals.has_copy_event,
            key_copy_path: signals.key_copy_path,
            key_copy_image: signals.key_copy_image,
            active_shift: signals.active_shift,
        });
        assert_eq!(decision.action, Some(CopyAction::Image));
    }

    #[test]
    fn integration_cmd_or_ctrl_shift_c_event_to_copy_path_action() {
        let events = vec![egui::Event::Key {
            key: egui::Key::C,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: copy_modifiers(true),
        }];
        let signals = collect_copy_shortcut_signals(&events, true);
        let decision = resolve_copy_action(CopyShortcutState {
            wants_keyboard_input: false,
            has_copy_event: signals.has_copy_event,
            key_copy_path: signals.key_copy_path,
            key_copy_image: signals.key_copy_image,
            active_shift: signals.active_shift,
        });
        assert_eq!(decision.action, Some(CopyAction::Path));
    }

    #[test]
    fn integration_copy_event_with_text_input_is_blocked() {
        let events = vec![egui::Event::Copy];
        let signals = collect_copy_shortcut_signals(&events, false);
        let decision = resolve_copy_action(CopyShortcutState {
            wants_keyboard_input: true,
            has_copy_event: signals.has_copy_event,
            key_copy_path: signals.key_copy_path,
            key_copy_image: signals.key_copy_image,
            active_shift: signals.active_shift,
        });
        assert_eq!(decision.action, None);
        assert!(decision.clear_hint);
        assert!(!decision.consume_copy_event);
    }
}
