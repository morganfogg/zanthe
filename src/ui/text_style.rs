#[derive(Clone, PartialEq)]
pub struct TextStyle {
    pub bold: bool,
    pub emphasis: bool,
    pub fixed_width: bool,
    pub reverse_video: bool,
}

impl Default for TextStyle {
    fn default() -> TextStyle {
        TextStyle {
            bold: false,
            emphasis: false,
            fixed_width: false,
            reverse_video: false,
        }
    }
}
