use super::data::EditorInner;
use editor::document::Document;

impl EditorInner {
    /// Применить замыкание к документу, перестроить кеш и пометить dirty.
    pub fn edit_doc<F>(&self, f: F)
    where
        F: FnOnce(&mut Document),
    {
        f(&mut self.doc.borrow_mut());

        let new_content = self.doc.borrow().content.clone();
        *self.cache.borrow_mut() = editor::markup::parse_document(&new_content);

        self.doc.borrow_mut().dirty = true;
    }
}
