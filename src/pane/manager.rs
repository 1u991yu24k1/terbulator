use crate::pane::{Pane, PaneId};
use crate::pane::layout::{Layout, Rect, SplitDirection};
use crate::utils::Result;
use std::collections::HashMap;

/// ペイン管理マネージャー
pub struct PaneManager {
    panes: HashMap<PaneId, Pane>,
    layout: Layout,
    active_pane_id: PaneId,
    broadcast_enabled: bool,
    shell: String,
    scrollback: usize,
}

impl PaneManager {
    /// 単一ペインで初期化
    pub fn new(cols: usize, rows: usize, scrollback: usize, shell: String) -> Result<Self> {
        let mut panes = HashMap::new();
        let initial_pane = Pane::new(0, cols, rows, scrollback, &shell)?;
        panes.insert(0, initial_pane);

        Ok(Self {
            panes,
            layout: Layout::new(),
            active_pane_id: 0,
            broadcast_enabled: false,
            shell,
            scrollback,
        })
    }

    /// アクティブなペインIDを取得
    pub fn active_pane_id(&self) -> PaneId {
        self.active_pane_id
    }

    /// アクティブなペインを取得
    pub fn active_pane(&self) -> Option<&Pane> {
        self.panes.get(&self.active_pane_id)
    }

    /// アクティブなペインを可変参照で取得
    pub fn active_pane_mut(&mut self) -> Option<&mut Pane> {
        self.panes.get_mut(&self.active_pane_id)
    }

    /// 指定されたIDのペインを取得
    pub fn pane(&self, pane_id: PaneId) -> Option<&Pane> {
        self.panes.get(&pane_id)
    }

    /// 指定されたIDのペインを可変参照で取得
    pub fn pane_mut(&mut self, pane_id: PaneId) -> Option<&mut Pane> {
        self.panes.get_mut(&pane_id)
    }

    /// 全ペインのイテレータ
    pub fn panes(&self) -> impl Iterator<Item = (&PaneId, &Pane)> {
        self.panes.iter()
    }

    /// レイアウトを取得
    pub fn layout(&self) -> &Layout {
        &self.layout
    }

    /// アクティブペインのRectを取得
    pub fn active_pane_rect(&self, window_rect: Rect) -> Option<Rect> {
        let rects = self.layout.calculate_rects(window_rect);
        rects.iter()
            .find(|(id, _)| *id == self.active_pane_id)
            .map(|(_, rect)| *rect)
    }

    /// Broadcastモードの状態を取得
    pub fn is_broadcast_enabled(&self) -> bool {
        self.broadcast_enabled
    }

    /// Broadcastモードを切り替え
    pub fn toggle_broadcast(&mut self) {
        self.broadcast_enabled = !self.broadcast_enabled;
        log::info!("Broadcast mode: {}", if self.broadcast_enabled { "enabled" } else { "disabled" });
    }

    /// アクティブペインを指定されたIDに設定
    pub fn set_active_pane(&mut self, pane_id: PaneId) -> bool {
        if self.panes.contains_key(&pane_id) {
            // 前のアクティブペインを非アクティブに
            if let Some(old_pane) = self.panes.get_mut(&self.active_pane_id) {
                old_pane.set_active(false);
            }

            // 新しいアクティブペインを設定
            self.active_pane_id = pane_id;
            if let Some(new_pane) = self.panes.get_mut(&pane_id) {
                new_pane.set_active(true);
            }

            log::debug!("Active pane changed to {}", pane_id);
            true
        } else {
            false
        }
    }

    /// アクティブペインを分割
    pub fn split_active_pane(&mut self, direction: SplitDirection, window_rect: Rect, cell_width: f32, cell_height: f32) -> Result<PaneId> {
        self.split_active_pane_with_ratio(direction, window_rect, cell_width, cell_height, 0.5)
    }

    /// アクティブペインを指定された比率で分割
    pub fn split_active_pane_with_ratio(&mut self, direction: SplitDirection, window_rect: Rect, cell_width: f32, cell_height: f32, ratio: f32) -> Result<PaneId> {
        let active_id = self.active_pane_id;

        // レイアウトツリーを分割（新しいペインIDが返される）
        let new_id = if let Some(id) = self.layout.split_pane_with_ratio(active_id, direction, ratio) {
            id
        } else {
            return Err(crate::utils::TerbulatorError::rendering("Failed to split pane in layout"));
        };

        // 新しいペインの矩形を計算
        let rects = self.layout.calculate_rects(window_rect);

        // 新しいペインのサイズを見つける
        if let Some((_, new_rect)) = rects.iter().find(|(id, _)| *id == new_id) {
            // 新しいペインを作成
            let cols = (new_rect.width as f32 / cell_width).max(1.0) as usize;
            let rows = (new_rect.height as f32 / cell_height).max(1.0) as usize;
            log::info!("Split active pane {}: new_id={}, cols={}, rows={}, rect={}x{}, cell={}x{}, shell={}",
                active_id, new_id, cols, rows, new_rect.width, new_rect.height, cell_width, cell_height, self.shell);

            let new_pane = match Pane::new(new_id, cols, rows, self.scrollback, &self.shell) {
                Ok(pane) => {
                    log::info!("Successfully created new pane {}", new_id);
                    pane
                }
                Err(e) => {
                    log::error!("Failed to create new pane {}: {}", new_id, e);
                    return Err(e);
                }
            };

            self.panes.insert(new_id, new_pane);

            // 全ペインをリサイズ
            self.resize_all_panes(window_rect, cell_width, cell_height)?;

            log::info!("Split pane {} in {:?} direction, created pane {}", active_id, direction, new_id);
            Ok(new_id)
        } else {
            Err(crate::utils::TerbulatorError::rendering("Failed to calculate new pane rect"))
        }
    }

    /// 指定されたペインを閉じる
    pub fn close_pane(&mut self, pane_id: PaneId, window_rect: Rect, cell_width: f32, cell_height: f32) -> Result<bool> {
        // 最後のペインは閉じられない
        if self.panes.len() <= 1 {
            log::warn!("Cannot close the last pane");
            return Ok(false);
        }

        // レイアウトから削除
        if !self.layout.remove_pane(pane_id) {
            log::warn!("Failed to remove pane {} from layout", pane_id);
            return Ok(false);
        }

        // ペインを削除
        self.panes.remove(&pane_id);

        // アクティブペインが削除された場合、次のペインをアクティブに
        if self.active_pane_id == pane_id {
            let next_id = *self.panes.keys().next().unwrap();
            self.set_active_pane(next_id);
        }

        // 全ペインをリサイズ
        self.resize_all_panes(window_rect, cell_width, cell_height)?;

        log::info!("Closed pane {}", pane_id);
        Ok(true)
    }

    /// アクティブペインを閉じる
    pub fn close_active_pane(&mut self, window_rect: Rect, cell_width: f32, cell_height: f32) -> Result<bool> {
        let active_id = self.active_pane_id;
        self.close_pane(active_id, window_rect, cell_width, cell_height)
    }

    /// 全ペインのPTY出力を処理
    /// 戻り値: (has_output, should_exit)
    /// - has_output: 何らかの出力があったか
    /// - should_exit: 全ペインが終了したためアプリケーションを終了すべきか
    pub fn process_all_pty_output(&mut self, window_rect: Rect, cell_width: f32, cell_height: f32) -> Result<(bool, bool)> {
        let mut has_any_output = false;
        let mut dead_panes = Vec::new();

        // PTY出力を処理し、終了したペインを収集
        for (pane_id, pane) in self.panes.iter_mut() {
            match pane.process_pty_output() {
                Ok(has_output) => {
                    if has_output {
                        log::trace!("Pane {} has output", pane_id);
                    }
                    has_any_output = has_any_output || has_output;
                }
                Err(e) => {
                    log::error!("Error processing PTY output for pane {}: {}", pane_id, e);
                }
            }

            // プロセスが終了しているかチェック
            if !pane.is_alive() {
                log::info!("Pane {} is not alive", pane_id);
                dead_panes.push(*pane_id);
            }
        }

        // 終了したペインを閉じる
        for pane_id in dead_panes {
            log::info!("Pane {} process exited, closing pane", pane_id);

            // 最後のペインの場合
            if self.panes.len() == 1 {
                log::info!("Last pane exited, application should exit");
                return Ok((has_any_output, true)); // アプリケーション終了
            }

            // ペインを閉じる（複数ある場合）
            if let Err(e) = self.close_pane(pane_id, window_rect, cell_width, cell_height) {
                log::error!("Failed to close dead pane {}: {}", pane_id, e);
            }
        }

        Ok((has_any_output, false))
    }

    /// 入力を送信（Broadcastモード対応）
    pub fn write_input(&self, data: &[u8]) -> Result<()> {
        if self.broadcast_enabled {
            // Broadcastモード: 全ペインに送信
            for pane in self.panes.values() {
                pane.write_input(data)?;
            }
            log::trace!("Broadcast input to {} panes: {} bytes", self.panes.len(), data.len());
        } else {
            // 通常モード: アクティブペインのみに送信
            if let Some(pane) = self.panes.get(&self.active_pane_id) {
                pane.write_input(data)?;
            }
        }
        Ok(())
    }

    /// ウィンドウリサイズ時に全ペインをリサイズ
    pub fn resize_all_panes(&mut self, window_rect: Rect, cell_width: f32, cell_height: f32) -> Result<()> {
        let rects = self.layout.calculate_rects(window_rect);

        log::debug!("Resizing all panes, window_rect: {}x{}, cell: {}x{}",
            window_rect.width, window_rect.height, cell_width, cell_height);

        for (pane_id, rect) in rects {
            if let Some(pane) = self.panes.get_mut(&pane_id) {
                // セルサイズから行列数を計算
                let cols = (rect.width as f32 / cell_width).max(1.0) as usize;
                let rows = (rect.height as f32 / cell_height).max(1.0) as usize;
                log::debug!("Resizing pane {} to {}x{} (rect: {}x{})", pane_id, cols, rows, rect.width, rect.height);
                pane.resize(cols, rows)?;
            } else {
                log::warn!("Pane {} not found in manager", pane_id);
            }
        }

        Ok(())
    }

    /// 次のペインにフォーカスを移動
    pub fn focus_next(&mut self) -> bool {
        let all_ids = self.layout.all_pane_ids();
        if all_ids.len() <= 1 {
            return false;
        }

        if let Some(current_idx) = all_ids.iter().position(|&id| id == self.active_pane_id) {
            let next_idx = (current_idx + 1) % all_ids.len();
            self.set_active_pane(all_ids[next_idx]);
            return true;
        }
        false
    }

    /// 前のペインにフォーカスを移動
    pub fn focus_prev(&mut self) -> bool {
        let all_ids = self.layout.all_pane_ids();
        if all_ids.len() <= 1 {
            return false;
        }

        if let Some(current_idx) = all_ids.iter().position(|&id| id == self.active_pane_id) {
            let prev_idx = if current_idx == 0 {
                all_ids.len() - 1
            } else {
                current_idx - 1
            };
            self.set_active_pane(all_ids[prev_idx]);
            return true;
        }
        false
    }

    /// 左のペインにフォーカスを移動
    pub fn focus_left(&mut self, window_rect: Rect) -> bool {
        self.focus_direction(window_rect, Direction::Left)
    }

    /// 右のペインにフォーカスを移動
    pub fn focus_right(&mut self, window_rect: Rect) -> bool {
        self.focus_direction(window_rect, Direction::Right)
    }

    /// 上のペインにフォーカスを移動
    pub fn focus_up(&mut self, window_rect: Rect) -> bool {
        self.focus_direction(window_rect, Direction::Up)
    }

    /// 下のペインにフォーカスを移動
    pub fn focus_down(&mut self, window_rect: Rect) -> bool {
        self.focus_direction(window_rect, Direction::Down)
    }

    /// 指定方向のペインにフォーカスを移動
    fn focus_direction(&mut self, window_rect: Rect, direction: Direction) -> bool {
        let rects = self.layout.calculate_rects(window_rect);

        // 現在のペインの矩形を取得
        let current_rect = match rects.iter().find(|(id, _)| *id == self.active_pane_id) {
            Some((_, rect)) => rect,
            None => return false,
        };

        // 現在のペインの中心座標
        let current_center_x = current_rect.x + current_rect.width / 2;
        let current_center_y = current_rect.y + current_rect.height / 2;

        // 指定方向で最も近いペインを探す
        let mut best_pane_id: Option<PaneId> = None;
        let mut best_distance = u32::MAX;

        for (pane_id, rect) in &rects {
            if *pane_id == self.active_pane_id {
                continue;
            }

            let center_x = rect.x + rect.width / 2;
            let center_y = rect.y + rect.height / 2;

            let is_in_direction = match direction {
                Direction::Left => center_x < current_center_x,
                Direction::Right => center_x > current_center_x,
                Direction::Up => center_y < current_center_y,
                Direction::Down => center_y > current_center_y,
            };

            if !is_in_direction {
                continue;
            }

            // 距離を計算（マンハッタン距離）
            let dx = (center_x as i32 - current_center_x as i32).abs() as u32;
            let dy = (center_y as i32 - current_center_y as i32).abs() as u32;
            let distance = dx + dy;

            if distance < best_distance {
                best_distance = distance;
                best_pane_id = Some(*pane_id);
            }
        }

        if let Some(pane_id) = best_pane_id {
            self.set_active_pane(pane_id);
            true
        } else {
            false
        }
    }

    /// 指定された位置で境界をドラッグして分割比率を更新
    pub fn update_border_at(&mut self, x: u32, y: u32, window_rect: Rect, cell_width: f32, cell_height: f32) -> Result<bool> {
        // 新しい比率を計算（マウス位置から）
        let window_width = window_rect.width as f32;
        let window_height = window_rect.height as f32;

        // マウス位置から比率を計算
        // まず境界を見つける
        if let Some((direction, _current_ratio)) = self.layout.find_border_at(x, y, window_rect) {
            let new_ratio = match direction {
                SplitDirection::Horizontal => {
                    // 水平分割: y座標から比率を計算
                    ((y - window_rect.y) as f32 / window_height).clamp(0.1, 0.9)
                }
                SplitDirection::Vertical => {
                    // 垂直分割: x座標から比率を計算
                    ((x - window_rect.x) as f32 / window_width).clamp(0.1, 0.9)
                }
            };

            // レイアウトの比率を更新
            if self.layout.update_split_ratio_at(x, y, window_rect, new_ratio) {
                // 全ペインをリサイズ
                self.resize_all_panes(window_rect, cell_width, cell_height)?;
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// マウス位置が境界の近くにあるかチェック
    pub fn is_near_border(&self, x: u32, y: u32, window_rect: Rect) -> bool {
        self.layout.find_border_at(x, y, window_rect).is_some()
    }
}

#[derive(Debug, Clone, Copy)]
enum Direction {
    Left,
    Right,
    Up,
    Down,
}
