use crate::editor::markup::Segment;

#[derive(Default, Clone, Debug)]
pub struct MarkupCache {
    pub segments: Vec<Segment>,
}
