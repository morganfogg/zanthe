#[derive(Clone, PartialEq, Eq, Debug, Copy, Default)]
pub struct TextStyle {
    pub bold: bool,
    pub emphasis: bool,
    pub fixed_width: bool,
    pub reverse_video: bool,
}
