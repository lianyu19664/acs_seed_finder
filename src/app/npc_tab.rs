use crate::{
    core::utils::find_chinese_collision,
    scanners::npc::{extract_all_sect_elders, GameData, SectData},
};
use eframe::egui;
use std::sync::Arc;

#[derive(Default)]
pub struct NpcTabState {
    pub target_seed: i32,
    pub status_msg: String,
    pub settings_path: String,
    pub sect_results: Vec<SectData>,
    pub game_data: Arc<GameData>,
}

impl NpcTabState {
    pub fn is_busy(&self) -> bool {
        false
    }

    pub fn update(&mut self, _c: &egui::Context) {
        // 由于是单次 Seed 的确定性内存推演，耗时极短，直接在同步上下文中完成，
        // 移除了原有的多线程 BackgroundTask 进度监控机制。
    }

    pub fn render(&mut self, ui: &mut egui::Ui) {
        egui::Grid::new("npc_cfg_grid")
            .num_columns(2)
            .spacing([40.0, 15.0])
            .show(ui, |ui| {
                ui.label("Settings 目录:");
                ui.horizontal(|u| {
                    if u.button("📂 选择").clicked() {
                        if let Some(p) = rfd::FileDialog::new().pick_folder() {
                            self.settings_path = p.display().to_string();
                            self.game_data = Arc::new(GameData::load_from_dir(&p));
                            self.status_msg = format!(
                                "✅ 解析成功! 衣:{} 材:{} 符:{} 具:{}",
                                self.game_data.clothes.len(),
                                self.game_data.stuffs.len(),
                                self.game_data.spells.len(),
                                self.game_data.tools.len()
                            );
                        }
                    }
                    u.label(if self.settings_path.is_empty() {
                        "待选择"
                    } else {
                        &self.settings_path
                    });
                });
                ui.end_row();

                ui.label("全局世界 Seed:");
                ui.horizontal(|u| {
                    u.add(egui::DragValue::new(&mut self.target_seed).speed(1));
                    if let Some(col) = find_chinese_collision(self.target_seed) {
                        u.label(egui::RichText::new(format!("({})", col)).color(egui::Color32::GRAY));
                    }
                });
                ui.end_row();
            });

        ui.add_space(20.0);

        if ui.button("👁️ 执行天道透视 (精确映射)").clicked() {
            self.sect_results = extract_all_sect_elders(self.target_seed, &self.game_data);
            self.status_msg = "✅ 13 门派双层级大能数据溯源完毕！".into();
        }

        if !self.status_msg.is_empty() {
            ui.label(egui::RichText::new(&self.status_msg).color(egui::Color32::GREEN));
        }

        ui.separator();
        ui.heading("📜 门派大能纪要");
        ui.add_space(10.0);

        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                for sect in &self.sect_results {
                    egui::CollapsingHeader::new(egui::RichText::new(format!("🏰 {}", sect.sect_name)).strong().size(16.0))
                        .default_open(false)
                        .show(ui, |ui| {
                            for (e_idx, elder) in sect.elders.iter().enumerate() {
                                // 修复 1.3: 剔除了导致 UI 组件字形解析渲染异常且缺乏回退机制的复合序列 Emoji，采用规整的 '★'，并闭合海量数据的预加载
                                egui::CollapsingHeader::new(format!("★ 大能 #{}: {} ({})", e_idx + 1, elder.name, elder.level_name))
                                    .default_open(false)
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            // 独立分区1：本命符箓
                                            ui.group(|ui| {
                                                // 修复 1.4: 从前端渲染树形面板彻底剥离残留的非正式 "(Scope B)" 测试性追踪标签
                                                ui.label(egui::RichText::new("✨ 佩戴符箓").color(egui::Color32::GOLD));
                                                ui.separator();
                                                for talisman in &elder.talismans {
                                                    ui.label(format!("• {}", talisman));
                                                }
                                            });

                                            // 独立分区2：杂项物品栏
                                            ui.group(|ui| {
                                                // 修复 1.4: 彻底剥离非正式的 "(Scope C)" 硬编码残留标签
                                                ui.label(egui::RichText::new("🎒 物品栏").color(egui::Color32::LIGHT_BLUE));
                                                ui.separator();
                                                for item in &elder.inventory {
                                                    ui.label(format!("• {}", item));
                                                }
                                            });
                                        });
                                    });
                                ui.add_space(5.0);
                            }
                        });
                }
            });
    }
}