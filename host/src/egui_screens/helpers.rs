//! egui UI 辅助函数与样式常量

pub const DARK_BG: egui::Color32 = egui::Color32::from_rgb(20, 20, 40);
pub const PANEL_BG: egui::Color32 = egui::Color32::from_rgb(25, 25, 50);
pub const GOLD: egui::Color32 = egui::Color32::from_rgb(220, 200, 160);

pub fn dark_frame() -> egui::Frame {
    egui::Frame::new().fill(DARK_BG).inner_margin(0.0)
}

pub fn panel_frame() -> egui::Frame {
    egui::Frame::new().fill(PANEL_BG).inner_margin(40.0)
}

/// 标准菜单按钮，返回是否被点击
pub fn menu_btn(ui: &mut egui::Ui, size: egui::Vec2, label: &str) -> bool {
    let clicked = ui
        .add_sized(
            size,
            egui::Button::new(egui::RichText::new(label).size(18.0)),
        )
        .clicked();
    ui.add_space(8.0);
    clicked
}
