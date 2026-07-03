use gtk4::prelude::*;
use gtk4::{DrawingArea, EventControllerKey, GestureClick};

pub struct EditorWidget {
    area: gtk4::DrawingArea,
    editor: Rc<RefCell<Editor>>,
}

impl EditorWidget {
    pub fn new() -> DrawingArea {
        let area = DrawingArea::builder()
            .hexpand(true)
            .vexpand(true)
            .focusable(true)
            .build();

        area.set_draw_func(|_, cr, width, height| {
            // фон приложения
            cr.set_source_rgb(0.086, 0.086, 0.102);
            cr.paint().unwrap();

            // карточка редактора
            let x = 32.0;
            let y = 32.0;
            let w = (width as f64) - 64.0;
            let h = (height as f64) - 64.0;
            let r = 12.0;

            rounded_rect(cr, x, y, w, h, r);

            cr.set_source_rgb(0.12, 0.12, 0.18);
            cr.fill_preserve().unwrap();

            cr.set_source_rgb(0.23, 0.23, 0.28);
            cr.set_line_width(1.0);
            cr.stroke().unwrap();
        });

        let click = GestureClick::new();
        click.connect_pressed(|_, _, _, _| {});
        area.add_controller(click);

        let key = EventControllerKey::new();
        key.connect_key_pressed(|_, _, _, _| false.into());
        area.add_controller(key);

        area
    }
}

fn rounded_rect(
    cr: &gtk4::cairo::Context,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    r: f64,
) {
    use std::f64::consts::{FRAC_PI_2, PI};

    cr.new_path();

    cr.arc(x + w - r, y + r, r, -FRAC_PI_2, 0.0);
    cr.arc(x + w - r, y + h - r, r, 0.0, FRAC_PI_2);
    cr.arc(x + r, y + h - r, r, FRAC_PI_2, PI);
    cr.arc(x + r, y + r, r, PI, PI * 1.5);

    cr.close_path();
}