use cartero_interop::FileSaveError;
use cartero_objects::Field;
use serde::{Serialize, Serializer};
use toml_edit::{
    visit_mut::{self, VisitMut},
    InlineTable, Item, KeyMut, Value,
};

use crate::{field_table_value::FieldTableValue, ToField};

pub(crate) fn alphabetical_field_table<T, S>(
    field_table: &Option<FieldTableValue<T>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    T: From<Field> + ToField + Clone + Serialize,
    S: Serializer,
{
    if let Some(field_table) = field_table {
        let table = field_table.sorted();
        table.serialize(serializer)
    } else {
        serializer.serialize_none()
    }
}

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
    BodyVariables,
    Header,
    Headers,
    InactiveParam,
    InactiveParams,
    Unknown,
    BodyVariable,
    Variables,
    Variable,
}

impl DocumentPointer {
    pub fn normalize(self, key: &str) -> Self {
        match (self, key) {
            (DocumentPointer::Root, "authorization") => DocumentPointer::Authorization,
            (DocumentPointer::Root, "body") => DocumentPointer::Body,
            (DocumentPointer::Root, "headers") => DocumentPointer::Headers,
            (DocumentPointer::Root, "inactive-params") => DocumentPointer::InactiveParams,
            (DocumentPointer::Root, "variables") => DocumentPointer::Variables,
            (DocumentPointer::Body, "variables") => DocumentPointer::BodyVariables,
            (DocumentPointer::Headers, _) => DocumentPointer::Header,
            (DocumentPointer::BodyVariables, _) => DocumentPointer::BodyVariable,
            (DocumentPointer::InactiveParams, _) => DocumentPointer::InactiveParam,
            (DocumentPointer::Variables, _) => DocumentPointer::Variable,
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
            DocumentPointer::Authorization
            | DocumentPointer::Body
            | DocumentPointer::BodyVariables
            | DocumentPointer::Headers
            | DocumentPointer::Variables
            | DocumentPointer::InactiveParams => {
                // If it's an inline table, convert into a regular table.
                if let Item::Value(Value::InlineTable(inline_table)) = node {
                    let inline_table = std::mem::replace(inline_table, InlineTable::new());
                    let table = inline_table.into_table();
                    *node = Item::Table(table);
                }
            }
            _ => {}
        }

        let old_pointer = self.pointer;
        self.pointer = key_node;
        visit_mut::visit_table_like_kv_mut(self, key, node);
        self.pointer = old_pointer;
    }

    fn visit_array_mut(&mut self, node: &mut toml_edit::Array) {
        match self.pointer {
            DocumentPointer::BodyVariable
            | DocumentPointer::Header
            | DocumentPointer::InactiveParam
            | DocumentPointer::Variable => {
                // Body variable arrays must have some indentation. When you use a custom
                // serializer, the default seems to be to print every value in the same line.
                node.set_trailing_comma(true);
                node.set_trailing("\n");
                for element in node.iter_mut() {
                    element.decor_mut().set_prefix("\n    ");
                    element.decor_mut().set_suffix("");
                }
            }
            _ => {}
        }
    }
}
