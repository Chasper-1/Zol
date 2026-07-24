use super::data::EditorInner;
use editor::document::Document;

impl EditorInner {
    /// Применить замыкание к документу, перестроить кеш и пометить dirty.
    ///
    /// Замыкание мутирует `doc` (через `doc.incremental.edit(...)` или через
    /// API-функции), после чего кеш сегментов перестраивается из `IncrementalDoc`.
    ///
    /// Для текстовых правок предпочитайте [`edit_doc_raw`] — он использует
    /// `edit_visible()` и viewport-кеш.
    pub fn edit_doc<F>(&self, f: F)
    where
        F: FnOnce(&mut Document),
    {
        f(&mut self.doc.borrow_mut());

        // Перестраиваем кеш (viewport-aware)
        let vp = self.viewport.get();
        let new_cache = {
            let doc_ref = self.doc.borrow();
            editor::markup::segmenter::incremental_to_cache_visible(&doc_ref.incremental, Some(&vp))
        };
        *self.cache.borrow_mut() = new_cache;

        self.doc.borrow_mut().dirty = true;
    }

    /// Применить текстовую правку с явными параметрами (from, to, text).
    ///
    /// Использует `IncrementalDoc::edit_visible()` — парсит только строки
    /// в viewport, а не весь документ.
    pub fn edit_doc_raw(&self, from: usize, to: usize, text: &str) {
        let vp = self.viewport.get();
        self.doc.borrow_mut().incremental.edit_visible(from, to, text, &vp);

        // Viewport-кеш (сегменты только для видимых строк)
        let new_cache = {
            let doc_ref = self.doc.borrow();
            editor::markup::segmenter::incremental_to_cache_visible(&doc_ref.incremental, Some(&vp))
        };
        *self.cache.borrow_mut() = new_cache;

        // Обновляем курсор
        let cursor = &self.doc.borrow().cursor;
        let _ = cursor; // замыкание ниже может мутировать doc
        self.doc.borrow_mut().dirty = true;
    }
}
