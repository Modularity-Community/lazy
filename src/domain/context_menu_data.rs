use iced::Point;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextTarget {
    Folder(i64),
    Request(i64),
}

#[derive(Debug, Clone, Copy)]
pub struct ContextMenuData {
    pub target: ContextTarget,
    pub position: Point,
}
