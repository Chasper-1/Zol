// src/webkit/editor.rs
pub fn generate_editor_html(css_styles: &str) -> String {
    format!(
        r#"
        <!DOCTYPE html>
        <html lang="ru">
        <head>
            <meta charset="UTF-8">
            <style>{}</style>
        </head>
        <body>
            <div class="workspace-grid">
                <div class="editor-card">
                    <div class="editor-content" contenteditable="true">
                        <h1>Новая заметка</h1>
                        <p>Пространство редактора готово к работе...</p>
                    </div>
                </div>
            </div>
        </body>
        </html>
        "#,
        css_styles
    )
}