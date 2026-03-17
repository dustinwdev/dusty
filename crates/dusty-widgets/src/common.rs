/// Label content that can be either static text or a reactive closure.
pub enum LabelContent {
    /// A fixed string label.
    Static(String),
    /// A reactive label that recomputes on each read.
    Dynamic(Box<dyn Fn() -> String>),
}
