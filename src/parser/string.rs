//! string handles parsing of GPX-spec strings.

use errors::*;
use std::io::Read;
use xml::reader::XmlEvent;

use parser::verify_starting_tag;
use parser::Context;

/// consume consumes a single string as tag content.
pub fn consume<R: Read>(context: &mut Context<R>, tagname: &'static str) -> Result<String> {
    let mut string: Option<String> = None;
    verify_starting_tag(context, tagname)?;

    for event in context.reader() {
        match event.chain_err(|| "error while parsing XML")? {
            XmlEvent::StartElement { ref name, .. } => {
                bail!(ErrorKind::InvalidChildElement(
                    name.local_name.clone(),
                    tagname
                ));
            }
            XmlEvent::Characters(content) => string = Some(content),
            XmlEvent::EndElement { ref name } => {
                ensure!(
                    name.local_name == tagname,
                    ErrorKind::InvalidClosingTag(name.local_name.clone(), tagname)
                );
                return string.ok_or("no content inside string".into());
            }
            _ => {}
        }
    }
    bail!(ErrorKind::MissingClosingTag(tagname));
}

#[cfg(test)]
mod tests {
    use std::io::BufReader;

    use super::consume;
    use GpxVersion;

    #[test]
    fn consume_simple_string() {
        let result = consume!("<string>hello world</string>", GpxVersion::Gpx11, "string");

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "hello world");
    }

    #[test]
    fn consume_new_tag() {
        // cannot start new tag inside string
        let result = consume!("<foo>bar<baz></baz></foo>", GpxVersion::Gpx11, "foo");

        assert!(result.is_err());
    }

    #[test]
    fn consume_start_tag() {
        // must have starting tag
        let result = consume!("bar</foo>", GpxVersion::Gpx11, "foo");

        assert!(result.is_err());
    }

    #[test]
    fn consume_end_tag() {
        // must have ending tag
        let result = consume!("<foo>bar", GpxVersion::Gpx11, "foo");

        assert!(result.is_err());
    }

    #[test]
    fn consume_no_body() {
        // must have string content
        let result = consume!("<foo></foo>", GpxVersion::Gpx11, "foo");

        assert!(result.is_err());
    }

    #[test]
    fn consume_different_ending_tag() {
        // this is just illegal
        let result = consume!("<foo></foobar>", GpxVersion::Gpx11, "foo");

        assert!(result.is_err());
    }
}
