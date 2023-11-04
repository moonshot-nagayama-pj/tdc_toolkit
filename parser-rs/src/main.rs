use std::fs::File;
use std::io::{self, Read};
mod parser;
use parser::parse_t2_ptu;

fn main() -> io::Result<()> {
    let path = "../sampledata/example-10mins-ch1-t2.ptu";
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    match parse_t2_ptu(&buffer[..]) {
        Ok((_, tghd)) => println!("success {:?}", tghd.events.len()),
        Err(err) => {
            println!("{}", err);
        }
    }
    Ok(())
}
