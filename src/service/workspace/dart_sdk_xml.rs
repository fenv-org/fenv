use anyhow::bail;
use log::debug;
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use std::io::Write;

#[derive(Debug, PartialEq, Eq)]
pub struct DartSdkXml {
    name: String,
    library: Library,
}

impl DartSdkXml {
    pub fn read<P: AsRef<std::path::Path>>(path: P) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Self::parse(&content)
    }

    pub fn parse(xml: &str) -> anyhow::Result<Self> {
        parse_xml(xml)
    }

    pub fn has_library(&self, url: &str) -> bool {
        self.library.entries.iter().any(|entry| {
            if let LibraryEntry::Classes(classes) = entry {
                classes.roots.iter().any(|root| root.url == url)
            } else {
                false
            }
        })
    }

    pub fn stringify(&self) -> String {
        let mut buf: Vec<u8> = Vec::new();
        writeln!(buf, "<component name=\"{}\">", self.name).unwrap();
        writeln!(buf, "  <library name=\"{}\">", self.library.name).unwrap();
        for entry in &self.library.entries {
            match entry {
                LibraryEntry::Classes(roots) => {
                    writeln!(buf, "    <CLASSES>").unwrap();
                    for root in &roots.roots {
                        writeln!(buf, "      <root url=\"{}\" />", root.url).unwrap()
                    }
                    writeln!(buf, "    </CLASSES>").unwrap();
                }
                LibraryEntry::Javadoc => writeln!(buf, "    <JAVADOC />").unwrap(),
                LibraryEntry::Sources => writeln!(buf, "    <SOURCES />").unwrap(),
            }
        }
        writeln!(buf, "  </library>").unwrap();
        writeln!(buf, "</component>").unwrap();
        String::from_utf8(buf).unwrap()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Library {
    name: String,
    entries: Vec<LibraryEntry>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum LibraryEntry {
    Classes(Classes),
    Javadoc,
    Sources,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Classes {
    roots: Vec<Root>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Root {
    url: String,
}

fn parse_xml(xml: &str) -> anyhow::Result<DartSdkXml> {
    let mut reader = Reader::from_str(xml);
    reader.trim_text(true);

    let mut buf = Vec::new();

    let mut component_name = String::new();
    let mut library_name = String::new();
    let mut library_entries: Vec<LibraryEntry> = vec![];
    let mut roots: Vec<Root> = vec![];

    loop {
        match reader.read_event_into(&mut buf) {
            Err(e) => bail!("Error at position {}: {:?}", reader.buffer_position(), e),

            Ok(Event::Eof) => {
                break Ok(DartSdkXml {
                    name: component_name,
                    library: Library {
                        name: library_name,
                        entries: library_entries,
                    },
                })
            }

            Ok(Event::Start(e)) => match e.name().as_ref() {
                b"component" => {
                    debug!("`component` tag found");
                    match e.try_get_attribute("name") {
                        Ok(Some(attr_name)) => {
                            component_name =
                                String::from_utf8(attr_name.value.into_owned()).unwrap();
                        }
                        Ok(_) => bail!("`name` attribute not found in `component` tag"),
                        Err(e) => bail!("`name` attribute not found in `component` tag: {}", e),
                    }
                }
                b"library" => {
                    debug!("`library` tag found");
                    match e.try_get_attribute("name") {
                        Ok(Some(attr_name)) => {
                            library_name = String::from_utf8(attr_name.value.into_owned()).unwrap();
                        }
                        Ok(_) => bail!("`name` attribute not found in `library` tag"),
                        Err(e) => bail!("`name` attribute not found in `library` tag: {}", e),
                    }
                }
                b"CLASSES" => {
                    debug!("`CLASSES` tag found");
                }

                _ => println!("Unknown tag: {:?}", e.name()),
            },

            Ok(Event::Empty(e)) => match e.name().as_ref() {
                b"root" => {
                    debug!("`root` tag found");
                    match e.try_get_attribute("url") {
                        Ok(Some(attr_name)) => {
                            roots.push(Root {
                                url: String::from_utf8(attr_name.value.into_owned()).unwrap(),
                            });
                        }
                        Ok(_) => bail!("`url` attribute not found in `root` tag"),
                        Err(e) => bail!("`url` attribute not found in `root` tag: {}", e),
                    }
                }
                b"JAVADOC" => {
                    debug!("`JAVADOC` tag found");
                    library_entries.push(LibraryEntry::Javadoc);
                }
                b"SOURCES" => {
                    debug!("`JAVADOC` tag found");
                    library_entries.push(LibraryEntry::Sources);
                }

                _ => println!("Unknown tag: {:?}", e.name()),
            },

            Ok(Event::End(e)) => match e.name().as_ref() {
                b"CLASSES" => {
                    debug!("`CLASSES` tag closed");
                    library_entries.push(LibraryEntry::Classes(Classes {
                        roots: roots.clone(),
                    }));
                }
                _ => (),
            },

            // There are several other `Event`s we do not consider here
            Ok(e) => println!("Unknown event: {:?}", e),
        }
        // if we don't keep a borrow elsewhere, we can clear the buffer to keep memory usage low
        buf.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::DartSdkXml;
    use crate::{
        service::workspace::dart_sdk_xml::{Classes, Library, LibraryEntry, Root},
        util::path_like::PathLike,
    };

    fn root_of(url: &str) -> Root {
        Root {
            url: url.to_string(),
        }
    }

    #[test]
    fn test_parsing() {
        let actual = DartSdkXml::read(
            &PathLike::from(std::env!("CARGO_MANIFEST_DIR"))
                .join("resources/test/Dart_SDK/sample.xml"),
        )
        .unwrap();
        let expected = DartSdkXml {
            name: "libraryTable".to_string(),
            library: Library {
                name: "Dart SDK".to_string(),
                entries: vec![
                    LibraryEntry::Classes(Classes {
                        roots: vec![
                            root_of(
                                "file://$USER_HOME$/.fenv/versions/3.7.12/bin/cache/dart-sdk/lib/async",
                            ),
                            root_of(
                                "file://$USER_HOME$/.fenv/versions/3.7.12/bin/cache/dart-sdk/lib/cli",
                            ),
                            root_of(
                                "file://$USER_HOME$/.fenv/versions/3.7.12/bin/cache/dart-sdk/lib/collection"
                            ),
                            root_of(
                                "file://$USER_HOME$/.fenv/versions/3.7.12/bin/cache/dart-sdk/lib/convert"
                            ),
                            root_of(
                                "file://$USER_HOME$/.fenv/versions/3.7.12/bin/cache/dart-sdk/lib/core"
                            ),
                            root_of(
                                "file://$USER_HOME$/.fenv/versions/3.7.12/bin/cache/dart-sdk/lib/developer"
                            ),
                            root_of(
                                "file://$USER_HOME$/.fenv/versions/3.7.12/bin/cache/dart-sdk/lib/ffi"
                            ),
                            root_of(
                                "file://$USER_HOME$/.fenv/versions/3.7.12/bin/cache/dart-sdk/lib/html"
                            ),
                            root_of(
                                "file://$USER_HOME$/.fenv/versions/3.7.12/bin/cache/dart-sdk/lib/indexed_db"
                            ),
                            root_of(
                                "file://$USER_HOME$/.fenv/versions/3.7.12/bin/cache/dart-sdk/lib/io"
                            ),
                            root_of(
                                "file://$USER_HOME$/.fenv/versions/3.7.12/bin/cache/dart-sdk/lib/isolate"
                            ),
                            root_of(
                                "file://$USER_HOME$/.fenv/versions/3.7.12/bin/cache/dart-sdk/lib/js"
                            ),
                            root_of(
                                "file://$USER_HOME$/.fenv/versions/3.7.12/bin/cache/dart-sdk/lib/js_util"
                            ),
                            root_of(
                                "file://$USER_HOME$/.fenv/versions/3.7.12/bin/cache/dart-sdk/lib/math"
                            ),
                            root_of(
                                "file://$USER_HOME$/.fenv/versions/3.7.12/bin/cache/dart-sdk/lib/mirrors"
                            ),
                            root_of(
                                "file://$USER_HOME$/.fenv/versions/3.7.12/bin/cache/dart-sdk/lib/svg"
                            ),
                            root_of(
                                "file://$USER_HOME$/.fenv/versions/3.7.12/bin/cache/dart-sdk/lib/typed_data"
                            ),
                            root_of(
                                "file://$USER_HOME$/.fenv/versions/3.7.12/bin/cache/dart-sdk/lib/web_audio"
                            ),
                            root_of(
                                "file://$USER_HOME$/.fenv/versions/3.7.12/bin/cache/dart-sdk/lib/web_gl"
                            )
                        ],
                    }),
                    LibraryEntry::Javadoc,
                    LibraryEntry::Sources,
                ],
            },
        };

        assert_eq!(actual, expected)
    }

    #[test]
    fn test_has_library() {
        let parsed = DartSdkXml::read(
            &PathLike::from(std::env!("CARGO_MANIFEST_DIR"))
                .join("resources/test/Dart_SDK/sample.xml"),
        )
        .unwrap();

        assert!(parsed
            .has_library("file://$USER_HOME$/.fenv/versions/3.7.12/bin/cache/dart-sdk/lib/core"));
        assert!(!parsed.has_library(
            "file://$USER_HOME$/.fenv/versions/3.7.12/bin/cache/dart-sdk/lib/invalid"
        ));
    }

    #[test]
    fn test_stringify() {
        let xml = PathLike::from(std::env!("CARGO_MANIFEST_DIR"))
            .join("resources/test/Dart_SDK/sample.xml")
            .read_to_string()
            .unwrap();

        let parsed = DartSdkXml::parse(&xml).unwrap();
        let stringified = parsed.stringify();

        assert_eq!(xml, stringified)
    }
}
