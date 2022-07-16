//! This example demonstrate how custom entities can be extracted from the DOCTYPE!,
//! and later use to decode text and attribute values.
//!
//! NB: this example is deliberately kept simple:
//! * it assumes that the XML file is UTF-8 encoded (custom_entities must only contain UTF-8 data)
//! * it only handles internal entities;
//! * the regex in this example is simple but brittle;
//! * it does not support the use of entities in entity declaration.

use std::collections::HashMap;

use quick_xml::events::Event;
use quick_xml::Reader;
use regex::bytes::Regex;

const DATA: &str = r#"

    <?xml version="1.0"?>
    <!DOCTYPE test [
    <!ENTITY msg "hello world" >
    ]>
    <test label="&msg;">&msg;</test>

"#;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut reader = Reader::from_str(DATA);
    reader.trim_text(true);

    let mut buf = Vec::new();
    let mut custom_entities: HashMap<String, String> = HashMap::new();
    let entity_re = Regex::new(r#"<!ENTITY\s+([^ \t\r\n]+)\s+"([^"]*)"\s*>"#)?;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::DocType(ref e)) => {
                for cap in entity_re.captures_iter(&e) {
                    custom_entities.insert(
                        String::from_utf8(cap[1].to_owned())?,
                        String::from_utf8(cap[2].to_owned())?,
                    );
                }
            }
            Ok(Event::Start(ref e)) => match e.name().as_ref() {
                b"test" => {
                    let attributes = e
                        .attributes()
                        .map(|a| {
                            a.unwrap()
                                .unescape_value_with(|ent| {
                                    custom_entities.get(ent).map(|s| s.as_str())
                                })
                                .unwrap()
                                .into_owned()
                        })
                        .collect::<Vec<String>>();
                    println!("attributes values: {:?}", attributes);
                }
                _ => (),
            },
            Ok(Event::Text(ref e)) => {
                let txt = e
                    .unescape_with(|ent| custom_entities.get(ent).map(|s| s.as_str()))
                    .unwrap();
                println!("text value: {}", txt);
            }
            Ok(Event::Eof) => break,
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
            _ => (),
        }
    }
    Ok(())
}
