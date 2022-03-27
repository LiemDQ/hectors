mod editor;
mod row;
mod highlight;
mod file;
mod screen;

use editor::Editor;
use file::File;

fn main() -> Result<(), std::io::Error> {
    let args : Vec<String> = std::env::args().collect();
    let file = if let Some(filename) = args.get(1) {
        File::open(filename)?
    } else {
        File::default()
    };

    Editor::new(file).unwrap().run();

    Ok(())
}
