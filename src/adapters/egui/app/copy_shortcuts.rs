//! 复制快捷键判定逻辑与信号聚合

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CopyAction {
    Image,
    Path,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct CopyShortcutState {
    pub(super) wants_keyboard_input: bool,
    pub(super) has_focused_widget: bool,
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
    pub(super) consume_shift_copy_key_event: bool,
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
    let block_shift_copy = should_block_shift_copy_in_focused_context(state);

    if state.wants_keyboard_input || state.has_focused_widget {
        if state.key_copy_path || state.key_copy_image || state.has_copy_event {
            return CopyDecision {
                action: None,
                clear_hint: true,
                consume_copy_event: block_shift_copy,
                consume_shift_copy_key_event: block_shift_copy,
            };
        }
        return CopyDecision {
            action: None,
            clear_hint: false,
            consume_copy_event: false,
            consume_shift_copy_key_event: false,
        };
    }

    let copy_path = state.key_copy_path || (state.has_copy_event && state.active_shift);
    if copy_path {
        return CopyDecision {
            action: Some(CopyAction::Path),
            clear_hint: false,
            consume_copy_event: false,
            consume_shift_copy_key_event: false,
        };
    }

    let copy_image = state.key_copy_image || (state.has_copy_event && !state.active_shift);
    if copy_image {
        return CopyDecision {
            action: Some(CopyAction::Image),
            clear_hint: false,
            consume_copy_event: false,
            consume_shift_copy_key_event: false,
        };
    }

    CopyDecision {
        action: None,
        clear_hint: false,
        consume_copy_event: false,
        consume_shift_copy_key_event: false,
    }
}

pub(super) fn should_block_shift_copy_in_focused_context(state: CopyShortcutState) -> bool {
    (state.wants_keyboard_input || state.has_focused_widget)
        && (state.key_copy_path || (state.has_copy_event && state.active_shift))
}

#[cfg(test)]
mod tests {
    use super::{
        collect_copy_shortcut_signals, resolve_copy_action, should_block_shift_copy_in_focused_context,
        CopyAction, CopyShortcutState,
    };

    fn copy_modifiers(shift: bool) -> egui::Modifiers {
        #[cfg(target_os = "macos")]
        {
            egui::Modifiers {
                shift,
                mac_cmd: true,
                ..Default::default()
            }
        }
        #[cfg(not(target_os = "macos"))]
        {
            egui::Modifiers {
                shift,
                ctrl: true,
                ..Default::default()
            }
        }
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
            has_focused_widget: false,
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
            has_focused_widget: false,
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
            has_focused_widget: true,
            has_copy_event: signals.has_copy_event,
            key_copy_path: signals.key_copy_path,
            key_copy_image: signals.key_copy_image,
            active_shift: signals.active_shift,
        });
        assert_eq!(decision.action, None);
        assert!(decision.clear_hint);
        assert!(!decision.consume_copy_event);
        assert!(!decision.consume_shift_copy_key_event);
    }

    #[test]
    fn integration_copy_event_without_key_triggers_copy_image_when_not_text_editing() {
        let events = vec![egui::Event::Copy];
        let signals = collect_copy_shortcut_signals(&events, false);
        let decision = resolve_copy_action(CopyShortcutState {
            wants_keyboard_input: false,
            has_focused_widget: false,
            has_copy_event: signals.has_copy_event,
            key_copy_path: signals.key_copy_path,
            key_copy_image: signals.key_copy_image,
            active_shift: signals.active_shift,
        });
        assert_eq!(decision.action, Some(CopyAction::Image));
        assert!(!decision.clear_hint);
        assert!(!decision.consume_copy_event);
        assert!(!decision.consume_shift_copy_key_event);
    }

    #[test]
    fn integration_cmd_shift_c_with_focused_widget_does_not_copy_path() {
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
            has_focused_widget: true,
            has_copy_event: signals.has_copy_event,
            key_copy_path: signals.key_copy_path,
            key_copy_image: signals.key_copy_image,
            active_shift: signals.active_shift,
        });
        assert_eq!(decision.action, None);
        assert!(decision.clear_hint);
        assert!(decision.consume_copy_event);
        assert!(decision.consume_shift_copy_key_event);
    }

    #[test]
    fn integration_copy_event_with_shift_and_focused_widget_is_consumed() {
        let events = vec![egui::Event::Copy];
        let signals = collect_copy_shortcut_signals(&events, true);
        let decision = resolve_copy_action(CopyShortcutState {
            wants_keyboard_input: false,
            has_focused_widget: true,
            has_copy_event: signals.has_copy_event,
            key_copy_path: signals.key_copy_path,
            key_copy_image: signals.key_copy_image,
            active_shift: signals.active_shift,
        });
        assert_eq!(decision.action, None);
        assert!(decision.clear_hint);
        assert!(decision.consume_copy_event);
        assert!(decision.consume_shift_copy_key_event);
    }

    #[test]
    fn helper_shift_copy_block_matrix() {
        assert!(should_block_shift_copy_in_focused_context(CopyShortcutState {
            wants_keyboard_input: true,
            has_focused_widget: true,
            has_copy_event: true,
            key_copy_path: false,
            key_copy_image: false,
            active_shift: true,
        }));
        assert!(should_block_shift_copy_in_focused_context(CopyShortcutState {
            wants_keyboard_input: false,
            has_focused_widget: true,
            has_copy_event: false,
            key_copy_path: true,
            key_copy_image: false,
            active_shift: true,
        }));
        assert!(!should_block_shift_copy_in_focused_context(CopyShortcutState {
            wants_keyboard_input: true,
            has_focused_widget: true,
            has_copy_event: true,
            key_copy_path: false,
            key_copy_image: false,
            active_shift: false,
        }));
    }
}
