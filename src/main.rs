use std::{env, error::Error, io::{self, Read}};

pub mod vertical;

use crate::vertical::*;

fn main() -> Result<(), Box<dyn Error>> {
    eprintln!("Reading from stdin");

    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let input_str = input.as_str();

    eprintln!("Running the parser");

    let contents = match Parser::new().parse(input_str) {
        Ok(contents) => contents,
        Err(err) => {
            eprintln!("{}", err.to_string());
            return Err("failed to parse code".into());
        }
    };

    eprintln!("Creating blueprint");

    let blueprint = contents.create_blueprint(
        &env::args().nth(2).unwrap_or("Blueprint".to_string()),
        Vec::new()
    )?;

    eprintln!("Encoding blueprint");

    blueprint.encode(io::stdout())?;

    eprintln!("All done!");

    Ok(())
}
