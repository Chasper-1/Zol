use super::data::EditorInner;
use editor::document::Document;

impl EditorInner {
    /// Применить замыкание к документу, перестроить кеш и пометить dirty.
    ///
    /// Замыкание мутирует `doc` (через `doc.incremental.edit(...)` или через
    /// API-функции), после чего кеш сегментов перестраивается из `IncrementalDoc`.
    pub fn edit_doc<F>(&self, f: F)
    where
        F: FnOnce(&mut Document),
    {
        f(&mut self.doc.borrow_mut());

        // Перестраиваем кеш из IncrementalDoc (без удержания RefCell)
        let new_cache = {
            let doc_ref = self.doc.borrow();
            editor::markup::segmenter::incremental_to_cache(&doc_ref.incremental)
        };
        *self.cache.borrow_mut() = new_cache;

        self.doc.borrow_mut().dirty = true;
    }
}
