use eframe::egui;
mod map_tab;
mod npc_tab;
pub mod task;
pub use {map_tab::MapTabState, npc_tab::NpcTabState};

#[derive(PartialEq, Clone, Copy, Default)]
pub enum AppTab {
    #[default]
    MapScanner,
    NpcScanner,
}
#[derive(Default)]
pub struct SeedFinderApp {
    pub app_tab: AppTab,
    pub map_tab: MapTabState,
    pub npc_tab: NpcTabState,
}

impl eframe::App for SeedFinderApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.map_tab.update(ctx);
        self.npc_tab.update(ctx);
        if self.map_tab.is_busy() || self.npc_tab.is_busy() {
            ctx.request_repaint();
        }
        egui::TopBottomPanel::top("tabs").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.app_tab, AppTab::MapScanner, "🌍 地图灵土雷达");
                ui.selectable_value(
                    &mut self.app_tab,
                    AppTab::NpcScanner,
                    "🔮 门派大能透视 (神符预测)",
                );
            })
        });
        egui::CentralPanel::default().show(ctx, |ui| match self.app_tab {
            AppTab::MapScanner => self.map_tab.render(ui),
            AppTab::NpcScanner => self.npc_tab.render(ui),
        });
    }
}
