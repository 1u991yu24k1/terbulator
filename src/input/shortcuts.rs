use winit::keyboard::{KeyCode, ModifiersState};

/// ショートカットアクション
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShortcutAction {
    /// ペイン分割（水平）
    SplitHorizontal,
    /// ペイン分割（垂直）
    SplitVertical,
    /// ペイン閉じる
    ClosePane,
    /// 左のペインに移動
    MoveFocusLeft,
    /// 下のペインに移動
    MoveFocusDown,
    /// 上のペインに移動
    MoveFocusUp,
    /// 右のペインに移動
    MoveFocusRight,
    /// 次のペインに移動
    MoveFocusNext,
    /// 前のペインに移動
    MoveFocusPrev,
    /// Broadcastモード切り替え
    ToggleBroadcast,
    /// Copy（選択範囲をクリップボードにコピー）
    Copy,
    /// Paste（クリップボードから貼り付け）
    Paste,
    /// フォントサイズを大きくする
    IncreaseFontSize,
    /// フォントサイズを小さくする
    DecreaseFontSize,
    /// マークモード切り替え
    ToggleMarkMode,
}

/// ショートカットハンドラー
pub struct ShortcutHandler;

impl ShortcutHandler {
    pub fn new() -> Self {
        Self
    }

    /// キー入力がショートカットに一致するか判定
    pub fn match_shortcut(
        &self,
        key_code: KeyCode,
        modifiers: ModifiersState,
    ) -> Option<ShortcutAction> {
        // Ctrl+Shift が押されているか確認
        let ctrl_shift = modifiers.control_key() && modifiers.shift_key();

        if ctrl_shift {
            match key_code {
                KeyCode::KeyH => Some(ShortcutAction::MoveFocusLeft),
                KeyCode::KeyJ => Some(ShortcutAction::MoveFocusDown),
                KeyCode::KeyK => Some(ShortcutAction::MoveFocusUp),
                KeyCode::KeyL => Some(ShortcutAction::MoveFocusRight),
                KeyCode::KeyN => Some(ShortcutAction::MoveFocusNext),
                KeyCode::KeyP => Some(ShortcutAction::MoveFocusPrev),
                KeyCode::KeyV => Some(ShortcutAction::SplitVertical),
                KeyCode::KeyS => Some(ShortcutAction::SplitHorizontal),
                KeyCode::KeyW => Some(ShortcutAction::ClosePane),
                KeyCode::KeyB => Some(ShortcutAction::ToggleBroadcast),
                KeyCode::KeyC => Some(ShortcutAction::Copy),
                _ => None,
            }
        } else if modifiers.control_key() && modifiers.shift_key() {
            // 既に上で処理済み
            None
        } else if modifiers.control_key() && key_code == KeyCode::KeyV {
            // Ctrl+V のみ（Shift なし）
            Some(ShortcutAction::Paste)
        } else if modifiers.alt_key() && modifiers.shift_key() {
            // Alt+Shift
            match key_code {
                KeyCode::KeyM => Some(ShortcutAction::ToggleMarkMode),
                _ => None,
            }
        } else if modifiers.control_key() {
            // Ctrl のみ（Shift なし）
            match key_code {
                KeyCode::Equal | KeyCode::NumpadAdd => Some(ShortcutAction::IncreaseFontSize),
                KeyCode::Minus | KeyCode::NumpadSubtract => Some(ShortcutAction::DecreaseFontSize),
                _ => None,
            }
        } else {
            None
        }
    }
}

impl Default for ShortcutHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pane_navigation_shortcuts() {
        let handler = ShortcutHandler::new();
        let mut modifiers = ModifiersState::empty();
        modifiers.set(ModifiersState::CONTROL, true);
        modifiers.set(ModifiersState::SHIFT, true);

        assert_eq!(
            handler.match_shortcut(KeyCode::KeyH, modifiers),
            Some(ShortcutAction::MoveFocusLeft)
        );
        assert_eq!(
            handler.match_shortcut(KeyCode::KeyJ, modifiers),
            Some(ShortcutAction::MoveFocusDown)
        );
        assert_eq!(
            handler.match_shortcut(KeyCode::KeyK, modifiers),
            Some(ShortcutAction::MoveFocusUp)
        );
        assert_eq!(
            handler.match_shortcut(KeyCode::KeyL, modifiers),
            Some(ShortcutAction::MoveFocusRight)
        );
        assert_eq!(
            handler.match_shortcut(KeyCode::KeyN, modifiers),
            Some(ShortcutAction::MoveFocusNext)
        );
        assert_eq!(
            handler.match_shortcut(KeyCode::KeyP, modifiers),
            Some(ShortcutAction::MoveFocusPrev)
        );
    }

    #[test]
    fn test_pane_management_shortcuts() {
        let handler = ShortcutHandler::new();
        let mut modifiers = ModifiersState::empty();
        modifiers.set(ModifiersState::CONTROL, true);
        modifiers.set(ModifiersState::SHIFT, true);

        assert_eq!(
            handler.match_shortcut(KeyCode::KeyV, modifiers),
            Some(ShortcutAction::SplitVertical)
        );
        assert_eq!(
            handler.match_shortcut(KeyCode::KeyS, modifiers),
            Some(ShortcutAction::SplitHorizontal)
        );
        assert_eq!(
            handler.match_shortcut(KeyCode::KeyW, modifiers),
            Some(ShortcutAction::ClosePane)
        );
        assert_eq!(
            handler.match_shortcut(KeyCode::KeyB, modifiers),
            Some(ShortcutAction::ToggleBroadcast)
        );
    }
}
