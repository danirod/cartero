use cartero_interop::FileSaveError;
use serde::Serialize;
use toml_edit::{visit_mut::VisitMut, InlineTable, Item, KeyMut, Value};

pub(crate) fn serialize<T>(v: &T) -> Result<String, FileSaveError>
where
    T: Serialize + ?Sized,
{
    let mut doc = toml_edit::ser::to_document(v)
        .map_err(|e| FileSaveError::SerializationError(Box::new(e)))?;
    let mut normalizer = SaveFileNormalizer::new();
    normalizer.visit_document_mut(&mut doc);

    Ok(doc.to_string())
}

#[derive(Copy, Clone, Debug)]
enum DocumentPointer {
    Root,
    Authorization,
    Body,
    Unknown,
}

impl DocumentPointer {
    pub fn normalize(self, key: &str) -> Self {
        match (self, key) {
            (DocumentPointer::Root, "authorization") => DocumentPointer::Authorization,
            (DocumentPointer::Root, "body") => DocumentPointer::Body,
            _ => DocumentPointer::Unknown,
        }
    }
}

#[derive(Debug)]
struct SaveFileNormalizer {
    pointer: DocumentPointer,
}

impl SaveFileNormalizer {
    fn new() -> Self {
        SaveFileNormalizer {
            pointer: DocumentPointer::Root,
        }
    }
}

impl VisitMut for SaveFileNormalizer {
    fn visit_table_like_kv_mut(&mut self, key: KeyMut<'_>, node: &mut Item) {
        let key_node = self.pointer.normalize(key.get());

        match key_node {
            DocumentPointer::Authorization | DocumentPointer::Body => {
                if let Item::Value(Value::InlineTable(inline_table)) = node {
                    let inline_table = std::mem::replace(inline_table, InlineTable::new());
                    let table = inline_table.into_table();
                    *node = Item::Table(table);
                }
            }
            _ => {}
        }
    }
}
