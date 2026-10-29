// FILE-CONTEXT
// ZWECK: Sicheres XML-Streaming-Parsing mit Event-Budgetierung und XXE-Schutz (P00B/P04).
// INVARIANTEN: Zero-Panic, LimitExceeded bei Budget/Tiefe, Corrupt bei XXE/DTD/Formatfehlern.
// STAND: TS:2026-10-31T00:00:00Z

use crate::error::{IngestError, LimitKind};
use crate::extractor::ResourceBudget;
use quick_xml::events::Event;
use quick_xml::reader::Reader;

/// Safe XML reader enforcing XML depth limits, event budget, and XXE protection.
pub struct SafeXmlReader<'a> {
    reader: Reader<&'a [u8]>,
    depth: u32,
    max_depth: u32,
}

impl<'a> SafeXmlReader<'a> {
    /// Creates a new `SafeXmlReader` from XML byte slice.
    pub fn new(bytes: &'a [u8], max_depth: u32) -> Self {
        let mut reader = Reader::from_reader(bytes);
        reader.config_mut().trim_text(false);
        reader.config_mut().expand_empty_elements = true;

        Self {
            reader,
            depth: 0,
            max_depth,
        }
    }

    /// Advances to the next XML event while charging the resource budget and enforcing depth limits.
    pub fn next_event(&mut self, budget: &ResourceBudget) -> Result<Event<'static>, IngestError> {
        budget.charge_xml_events(1)?;

        let mut buf = Vec::new();
        let event = self
            .reader
            .read_event_into(&mut buf)
            .map_err(|e| IngestError::Corrupt {
                format: None,
                detail: format!("XML Parse-Fehler: {e}"),
            })?;

        match &event {
            Event::Start(_) => {
                self.depth = self.depth.saturating_add(1);
                if self.depth > self.max_depth {
                    return Err(IngestError::LimitExceeded {
                        kind: LimitKind::XmlDepth,
                        limit: u64::from(self.max_depth),
                        observed: u64::from(self.depth),
                    });
                }
            }
            Event::End(_) => {
                self.depth = self.depth.saturating_sub(1);
            }
            Event::DocType(dt) => {
                // Reject DTD external entity declarations or entity expansion (XXE protection)
                let dt_bytes = dt.as_ref();
                if dt_bytes
                    .windows(6)
                    .any(|w| w.eq_ignore_ascii_case(b"ENTITY"))
                    || dt_bytes
                        .windows(6)
                        .any(|w| w.eq_ignore_ascii_case(b"SYSTEM"))
                    || dt_bytes
                        .windows(6)
                        .any(|w| w.eq_ignore_ascii_case(b"PUBLIC"))
                {
                    return Err(IngestError::Corrupt {
                        format: None,
                        detail: "XXE-Sicherheitsverstoß: Externe DTDs/Entitäten sind verboten"
                            .into(),
                    });
                }
            }
            _ => {}
        }

        Ok(event.into_owned())
    }
}
