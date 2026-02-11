use crate::pane::PaneId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitDirection {
    Horizontal,
    Vertical,
}

/// ペインの矩形領域
#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl Rect {
    pub fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self { x, y, width, height }
    }
}

/// レイアウトツリーのノード
#[derive(Debug, Clone)]
pub enum LayoutNode {
    Leaf {
        pane_id: PaneId,
    },
    Branch {
        direction: SplitDirection,
        ratio: f32, // 0.0-1.0, 最初の子が占める割合
        first: Box<LayoutNode>,
        second: Box<LayoutNode>,
    },
}

/// ペインのレイアウト管理
pub struct Layout {
    root: LayoutNode,
    next_id: PaneId,
}

impl Layout {
    /// 単一ペインで初期化
    pub fn new() -> Self {
        Self {
            root: LayoutNode::Leaf { pane_id: 0 },
            next_id: 1,
        }
    }

    /// ルートノードを取得
    pub fn root(&self) -> &LayoutNode {
        &self.root
    }

    /// 次のペインIDを生成
    pub fn next_id(&mut self) -> PaneId {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// 指定されたペインを分割
    pub fn split_pane(&mut self, pane_id: PaneId, direction: SplitDirection) -> Option<PaneId> {
        self.split_pane_with_ratio(pane_id, direction, 0.5)
    }

    /// 指定されたペインを指定された比率で分割
    pub fn split_pane_with_ratio(&mut self, pane_id: PaneId, direction: SplitDirection, ratio: f32) -> Option<PaneId> {
        let new_id = self.next_id();

        if Self::split_node(&mut self.root, pane_id, direction, new_id, ratio) {
            Some(new_id)
        } else {
            None
        }
    }

    fn split_node(
        node: &mut LayoutNode,
        target_id: PaneId,
        direction: SplitDirection,
        new_id: PaneId,
        ratio: f32,
    ) -> bool {
        match node {
            LayoutNode::Leaf { pane_id } if *pane_id == target_id => {
                // このノードを分割
                let old_leaf = LayoutNode::Leaf { pane_id: *pane_id };
                let new_leaf = LayoutNode::Leaf { pane_id: new_id };

                *node = LayoutNode::Branch {
                    direction,
                    ratio, // 指定された比率で分割
                    first: Box::new(old_leaf),
                    second: Box::new(new_leaf),
                };
                true
            }
            LayoutNode::Branch { first, second, .. } => {
                // 子ノードを再帰的に探す
                Self::split_node(first, target_id, direction, new_id, ratio)
                    || Self::split_node(second, target_id, direction, new_id, ratio)
            }
            _ => false,
        }
    }

    /// 指定されたペインを削除（兄弟ノードで置き換え）
    pub fn remove_pane(&mut self, pane_id: PaneId) -> bool {
        // ルートが削除対象の場合は削除できない（最後のペイン）
        if let LayoutNode::Leaf { pane_id: root_id } = self.root {
            if root_id == pane_id {
                return false;
            }
        }

        Self::remove_node(&mut self.root, pane_id)
    }

    fn remove_node(node: &mut LayoutNode, target_id: PaneId) -> bool {
        if let LayoutNode::Branch { first, second, .. } = node {
            // 子が削除対象か確認
            if let LayoutNode::Leaf { pane_id } = **first {
                if pane_id == target_id {
                    // firstを削除、secondで置き換え
                    *node = (**second).clone();
                    return true;
                }
            }

            if let LayoutNode::Leaf { pane_id } = **second {
                if pane_id == target_id {
                    // secondを削除、firstで置き換え
                    *node = (**first).clone();
                    return true;
                }
            }

            // 再帰的に探す
            Self::remove_node(first, target_id) || Self::remove_node(second, target_id)
        } else {
            false
        }
    }

    /// レイアウトを計算して各ペインの矩形を返す
    pub fn calculate_rects(&self, window_rect: Rect) -> Vec<(PaneId, Rect)> {
        let mut rects = Vec::new();
        self.calculate_node_rects(&self.root, window_rect, &mut rects);
        rects
    }

    fn calculate_node_rects(
        &self,
        node: &LayoutNode,
        rect: Rect,
        rects: &mut Vec<(PaneId, Rect)>,
    ) {
        match node {
            LayoutNode::Leaf { pane_id } => {
                rects.push((*pane_id, rect));
            }
            LayoutNode::Branch {
                direction,
                ratio,
                first,
                second,
            } => {
                let (first_rect, second_rect) = match direction {
                    SplitDirection::Horizontal => {
                        // 水平分割（上下）
                        let first_height = (rect.height as f32 * ratio) as u32;
                        let second_height = rect.height.saturating_sub(first_height);

                        let first_rect = Rect::new(rect.x, rect.y, rect.width, first_height);
                        let second_rect = Rect::new(
                            rect.x,
                            rect.y + first_height,
                            rect.width,
                            second_height,
                        );

                        (first_rect, second_rect)
                    }
                    SplitDirection::Vertical => {
                        // 垂直分割（左右）
                        let first_width = (rect.width as f32 * ratio) as u32;
                        let second_width = rect.width.saturating_sub(first_width);

                        let first_rect = Rect::new(rect.x, rect.y, first_width, rect.height);
                        let second_rect = Rect::new(
                            rect.x + first_width,
                            rect.y,
                            second_width,
                            rect.height,
                        );

                        (first_rect, second_rect)
                    }
                };

                self.calculate_node_rects(first, first_rect, rects);
                self.calculate_node_rects(second, second_rect, rects);
            }
        }
    }

    /// 全ペインIDを取得
    pub fn all_pane_ids(&self) -> Vec<PaneId> {
        let mut ids = Vec::new();
        self.collect_pane_ids(&self.root, &mut ids);
        ids
    }

    fn collect_pane_ids(&self, node: &LayoutNode, ids: &mut Vec<PaneId>) {
        match node {
            LayoutNode::Leaf { pane_id } => {
                ids.push(*pane_id);
            }
            LayoutNode::Branch { first, second, .. } => {
                self.collect_pane_ids(first, ids);
                self.collect_pane_ids(second, ids);
            }
        }
    }

    /// 指定された境界を見つけて比率を更新
    /// 境界の位置（x, y）と方向に基づいて対応するBranchノードのratioを更新
    pub fn update_split_ratio_at(&mut self, x: u32, y: u32, window_rect: Rect, new_ratio: f32) -> bool {
        Self::update_ratio_in_node(&mut self.root, x, y, window_rect, new_ratio)
    }

    fn update_ratio_in_node(
        node: &mut LayoutNode,
        x: u32,
        y: u32,
        rect: Rect,
        new_ratio: f32,
    ) -> bool {
        match node {
            LayoutNode::Leaf { .. } => false,
            LayoutNode::Branch {
                direction,
                ratio,
                first,
                second,
            } => {
                // 現在のノードの分割境界を計算
                let _boundary_pos = match direction {
                    SplitDirection::Horizontal => {
                        let split_y = rect.y + (rect.height as f32 * *ratio) as u32;
                        // マウスがこの境界の近くにあるか確認（±10ピクセル）
                        if y >= split_y.saturating_sub(10) && y <= split_y + 10 {
                            *ratio = new_ratio.clamp(0.1, 0.9);
                            return true;
                        }
                        split_y
                    }
                    SplitDirection::Vertical => {
                        let split_x = rect.x + (rect.width as f32 * *ratio) as u32;
                        // マウスがこの境界の近くにあるか確認（±10ピクセル）
                        if x >= split_x.saturating_sub(10) && x <= split_x + 10 {
                            *ratio = new_ratio.clamp(0.1, 0.9);
                            return true;
                        }
                        split_x
                    }
                };

                // 子ノードを再帰的に探す
                let (first_rect, second_rect) = match direction {
                    SplitDirection::Horizontal => {
                        let first_height = (rect.height as f32 * *ratio) as u32;
                        let second_height = rect.height.saturating_sub(first_height);

                        let first_rect = Rect::new(rect.x, rect.y, rect.width, first_height);
                        let second_rect = Rect::new(
                            rect.x,
                            rect.y + first_height,
                            rect.width,
                            second_height,
                        );

                        (first_rect, second_rect)
                    }
                    SplitDirection::Vertical => {
                        let first_width = (rect.width as f32 * *ratio) as u32;
                        let second_width = rect.width.saturating_sub(first_width);

                        let first_rect = Rect::new(rect.x, rect.y, first_width, rect.height);
                        let second_rect = Rect::new(
                            rect.x + first_width,
                            rect.y,
                            second_width,
                            rect.height,
                        );

                        (first_rect, second_rect)
                    }
                };

                Self::update_ratio_in_node(first, x, y, first_rect, new_ratio)
                    || Self::update_ratio_in_node(second, x, y, second_rect, new_ratio)
            }
        }
    }

    /// 指定された位置（x, y）が境界線の近くかどうかを判定
    /// 境界線の近くであれば、その境界の情報（方向、現在の比率）を返す
    pub fn find_border_at(&self, x: u32, y: u32, window_rect: Rect) -> Option<(SplitDirection, f32)> {
        Self::find_border_in_node(&self.root, x, y, window_rect)
    }

    fn find_border_in_node(
        node: &LayoutNode,
        x: u32,
        y: u32,
        rect: Rect,
    ) -> Option<(SplitDirection, f32)> {
        match node {
            LayoutNode::Leaf { .. } => None,
            LayoutNode::Branch {
                direction,
                ratio,
                first,
                second,
            } => {
                // 現在のノードの分割境界を計算
                let is_near_boundary = match direction {
                    SplitDirection::Horizontal => {
                        let split_y = rect.y + (rect.height as f32 * *ratio) as u32;
                        y >= split_y.saturating_sub(10) && y <= split_y + 10
                    }
                    SplitDirection::Vertical => {
                        let split_x = rect.x + (rect.width as f32 * *ratio) as u32;
                        x >= split_x.saturating_sub(10) && x <= split_x + 10
                    }
                };

                if is_near_boundary {
                    return Some((*direction, *ratio));
                }

                // 子ノードを再帰的に探す
                let (first_rect, second_rect) = match direction {
                    SplitDirection::Horizontal => {
                        let first_height = (rect.height as f32 * *ratio) as u32;
                        let second_height = rect.height.saturating_sub(first_height);

                        let first_rect = Rect::new(rect.x, rect.y, rect.width, first_height);
                        let second_rect = Rect::new(
                            rect.x,
                            rect.y + first_height,
                            rect.width,
                            second_height,
                        );

                        (first_rect, second_rect)
                    }
                    SplitDirection::Vertical => {
                        let first_width = (rect.width as f32 * *ratio) as u32;
                        let second_width = rect.width.saturating_sub(first_width);

                        let first_rect = Rect::new(rect.x, rect.y, first_width, rect.height);
                        let second_rect = Rect::new(
                            rect.x + first_width,
                            rect.y,
                            second_width,
                            rect.height,
                        );

                        (first_rect, second_rect)
                    }
                };

                Self::find_border_in_node(first, x, y, first_rect)
                    .or_else(|| Self::find_border_in_node(second, x, y, second_rect))
            }
        }
    }
}

impl Default for Layout {
    fn default() -> Self {
        Self::new()
    }
}
