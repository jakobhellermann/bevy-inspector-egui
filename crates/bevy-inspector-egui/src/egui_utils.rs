use egui::FontId;

pub fn layout_job(text: &[(FontId, &str)]) -> egui::epaint::text::LayoutJob {
    let mut job = egui::epaint::text::LayoutJob::default();
    for (font_id, text) in text {
        job.append(
            text,
            0.0,
            egui::TextFormat {
                font_id: font_id.clone(),
                ..Default::default()
            },
        );
    }
    job
}

pub fn label_button(ui: &mut egui::Ui, text: &str, text_color: egui::Color32) -> bool {
    ui.add(egui::Button::new(
        egui::RichText::new(text).color(text_color),
    ))
    .clicked()
}

struct IconButton {
    rect: egui::Rect,
    response: egui::Response,
    painter: egui::Painter,
    stroke: egui::Stroke,
}
impl IconButton {
    fn new(ui: &mut egui::Ui) -> Self {
        let button_size = egui::Vec2::splat(ui.spacing().icon_width);
        let (response, painter) = ui.allocate_painter(button_size, egui::Sense::click());
        let visuals = if ui.is_enabled() {
            ui.style().interact(&response)
        } else {
            ui.style().noninteractive()
        };
        let rect = response.rect.shrink(2.0).expand(visuals.expansion);
        let stroke = visuals.fg_stroke;
        IconButton {
            rect,
            response,
            painter,
            stroke,
        }
    }
    fn line(&self, points: [egui::Pos2; 2]) {
        self.painter.line_segment(points, self.stroke);
    }
    fn add_button(self) -> egui::Response {
        // paints |
        self.line([self.rect.center_top(), self.rect.center_bottom()]);
        // paints -
        self.line([self.rect.left_center(), self.rect.right_center()]);
        self.response
    }
    fn remove_button(self) -> egui::Response {
        // paints -
        self.line([self.rect.left_center(), self.rect.right_center()]);
        self.response
    }
    fn up_button(self) -> egui::Response {
        // paints |
        self.line([self.rect.center_top(), self.rect.center_bottom()]);
        // paints /
        self.line([self.rect.center_top(), self.rect.left_center()]);
        // paints \
        self.line([self.rect.center_top(), self.rect.right_center()]);
        self.response
    }
    fn down_button(self) -> egui::Response {
        // paints |
        self.line([self.rect.center_top(), self.rect.center_bottom()]);
        // paints \
        self.line([self.rect.left_center(), self.rect.center_bottom()]);
        // paints /
        self.line([self.rect.right_center(), self.rect.center_bottom()]);
        self.response
    }
}

pub fn add_button(ui: &mut egui::Ui) -> egui::Response {
    IconButton::new(ui).add_button()
}

pub fn remove_button(ui: &mut egui::Ui) -> egui::Response {
    IconButton::new(ui).remove_button()
}

pub fn up_button(ui: &mut egui::Ui) -> egui::Response {
    IconButton::new(ui).up_button()
}

pub fn down_button(ui: &mut egui::Ui) -> egui::Response {
    IconButton::new(ui).down_button()
}
