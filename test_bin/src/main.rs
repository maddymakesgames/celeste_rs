use celeste_rs::saves::SaveData;
use std::{fs::OpenOptions, io::BufReader};

fn main() -> std::io::Result<()> {
    let mut args = std::env::args();
    args.next().unwrap();
    let path = args.next().unwrap();
    println!("{path}");
    let file = OpenOptions::new().read(true).write(false).open(path)?;
    let document = &mut quick_xml::de::Deserializer::from_reader(BufReader::new(file));
    let document: SaveData = match serde_path_to_error::deserialize(document) {
        Ok(a) => a,
        Err(e) => {
            println!("{}", e.path());
            println!("{e:?}");
            return Ok(());
        }
    };

    let xml = document.to_string().unwrap();
    std::fs::write("./test.xml", xml).unwrap();
    Ok(())
}
