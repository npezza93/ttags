use ttags::App;
use std::error::Error;
use std::process::exit;

fn main() -> Result<(), Box<dyn Error>> {
    exit(App::run()?);
}
