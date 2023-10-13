use std::error::Error;
use std::process::exit;
use ttags::App;

fn main() -> Result<(), Box<dyn Error>> {
    exit(App::run()?);
}
