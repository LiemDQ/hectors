use termion::color;


#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Highlight {
    None,
    Normal,
    String,
    Character,
    Comment,
    MlComment,
    Keyword1,
    Keyword2,
    Number,
    Match,
    Caps
}

impl Highlight {
    pub fn to_true_color(self) -> impl color::Color {
        match self {
            //these values are taken from the konsole breathe color palette
            Highlight::Number => color::Rgb(237, 21, 21),
            Highlight::Match => color::Rgb(68, 133, 58),
            Highlight::String => color::Rgb(246, 116, 0),
            Highlight::Character => color::Rgb(29, 153, 243),
            Highlight::Comment | Highlight::MlComment => color::Rgb(61, 174, 233),
            Highlight::Keyword1 => color::Rgb(155, 89, 182),
            Highlight::Keyword2 => color::Rgb(253, 188, 75),
            _ => color::Rgb(23, 168, 139),
        }
    }

}