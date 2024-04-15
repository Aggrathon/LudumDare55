use bevy::prelude::*;
use bevy_egui::egui::{Align2, Color32, Frame, Layout, RichText, Rounding, Vec2, Widget};
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiSettings};
use enum_iterator::{all, cardinality};

use crate::level::{GameStats, Gameplay, Level, LevelLocal};
use crate::unit::{Spawner, UnitPrefab};

fn main_menu(mut contexts: EguiContexts, mut next_level: ResMut<NextState<Level>>) {
    contexts.ctx_mut().set_visuals(egui::Visuals::light());
    let height = contexts.ctx_mut().available_rect().max.y;
    egui::TopBottomPanel::top("top")
        .exact_height(height * 0.4)
        .frame(Frame::none())
        .show_separator_line(false)
        .show(contexts.ctx_mut(), |ui| {
            ui.with_layout(Layout::bottom_up(egui::Align::Center), |ui| {
                ui.heading(
                    RichText::new("The Lazy Archdemon")
                        .size(80.0)
                        .color(Color32::BLACK)
                        .strong(),
                );
            });
        });
    egui::TopBottomPanel::bottom("bottom")
        .exact_height(height * 0.6)
        .frame(Frame::none())
        .show_separator_line(false)
        .show(contexts.ctx_mut(), |ui| {
            ui.with_layout(Layout::top_down(egui::Align::Center), |ui| {
                ui.add_space(10.0);
                ui.label(
                    RichText::new("The archdemon has finally decided to invade earth!")
                        .size(28.0)
                        .color(Color32::BLACK),
                );
                ui.label(
                    RichText::new("But actually managing an invasion is far below their stature.")
                        .size(28.0)
                        .color(Color32::BLACK),
                );
                ui.label(
                    RichText::new("So, you have been assigned to oversee the summoning circles.")
                        .size(28.0)
                        .color(Color32::BLACK),
                );
                ui.label(
                    RichText::new(
                        "You even get a \"finders fee\" for every soul send back to hell!",
                    )
                    .size(28.0)
                    .color(Color32::BLACK),
                );
                ui.add_space(10.0);
                let button =
                    egui::Button::new(RichText::new("Play").size(40.0).color(Color32::BLACK))
                        .rounding(Rounding::from(5.0))
                        .min_size(Vec2::new(300.0, 30.0));
                if button.ui(ui).clicked() {
                    next_level.set(Level::Level01);
                }
            });
        });
}

fn game_ui(
    mut commands: Commands,
    mut contexts: EguiContexts,
    mut spawners: Query<&mut Spawner>,
    time: Res<Time>,
    mut stats: ResMut<GameStats>,
    mut next: ResMut<NextState<Level>>,
) {
    contexts.ctx_mut().set_visuals(egui::Visuals::dark());
    egui::SidePanel::left("left")
        .resizable(false)
        .show(contexts.ctx_mut(), |ui| {
            ui.heading("Summoning circles");
            ui.separator();
            for (i, mut s) in spawners.iter_mut().enumerate() {
                let mut prefab = s.prefab;
                egui::ComboBox::from_id_source(i)
                    .selected_text(s.prefab.name())
                    .width(ui.available_width())
                    .show_ui(ui, |ui| {
                        for p in all::<UnitPrefab>().take(stats.upgrade_level as usize + 1) {
                            ui.selectable_value(&mut prefab, p, p.name());
                        }
                    });
                if s.prefab != prefab {
                    s.prefab = prefab;
                }
                let mut number = s.number;
                ui.set_width(ui.available_width());
                egui::Slider::new(&mut number, 1..=9).integer().ui(ui);
                if number != s.number {
                    s.number = number;
                }
                egui::ProgressBar::new(s.progress(time.elapsed(), stats.upgrade_speed))
                    .desired_width(ui.available_width())
                    .ui(ui);
                ui.separator();
            }
        });
    egui::TopBottomPanel::bottom("bottom")
        .resizable(false)
        .show(contexts.ctx_mut(), |ui| {
            ui.horizontal(|ui| {
                let width = ui.available_width() / 3.0 - 10.0;
                ui.add(
                    egui::ProgressBar::new(
                        (time.elapsed() - stats.start_time).as_secs_f32()
                            / (stats.time_limit() - stats.start_time).as_secs_f32(),
                    )
                    .text("Archdemon boredom")
                    .desired_width(width)
                    .fill(Color32::from_rgb(180, 100, 0)),
                );
                ui.add(
                    egui::ProgressBar::new(stats.souls_current as f32 / stats.souls_next as f32)
                        .text("Recycled souls")
                        .desired_width(width)
                        .fill(Color32::from_rgb(140, 0, 210)),
                );
                ui.add(
                    egui::ProgressBar::new(
                        stats.defender_morale as f32 / GameStats::MAX_MORALE as f32,
                    )
                    .text("Defenders' morale")
                    .desired_width(width)
                    .fill(Color32::from_rgb(180, 0, 0)),
                );
            })
        });
    if stats.defender_morale == 0 {
        egui::CentralPanel::default()
            .frame(Frame {
                fill: Color32::from_white_alpha(128),
                ..default()
            })
            .show(contexts.ctx_mut(), |_| {});
        egui::Window::new("You have broken though the defences!")
            .resizable(false)
            .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
            .show(contexts.ctx_mut(), |ui| {
                ui.horizontal(|ui| {
                    let width = ui.available_width() / 2.0 - 5.0;
                    if egui::Button::new("Replay")
                        .min_size(Vec2::new(width, 10.0))
                        .ui(ui)
                        .clicked()
                    {
                        next.set(Level::Reload);
                    }
                    if egui::Button::new("Next")
                        .min_size(Vec2::new(width, 10.0))
                        .ui(ui)
                        .clicked()
                    {
                        next.set(Level::Next);
                    }
                })
            });
    } else if stats.time_limit() < time.elapsed() {
        egui::CentralPanel::default()
            .frame(Frame {
                fill: Color32::from_black_alpha(128),
                ..default()
            })
            .show(contexts.ctx_mut(), |_| {});
        egui::Window::new("The archdemon got bored and ate your soul!")
            .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
            .resizable(false)
            .show(contexts.ctx_mut(), |ui| {
                ui.horizontal(|ui| {
                    let width = ui.available_width() / 2.0 - 5.0;
                    if egui::Button::new("Retry")
                        .min_size(Vec2::new(width, 10.0))
                        .ui(ui)
                        .clicked()
                    {
                        next.set(Level::Reload);
                    }
                    if egui::Button::new("Skip")
                        .min_size(Vec2::new(width, 10.0))
                        .ui(ui)
                        .clicked()
                    {
                        next.set(Level::Next);
                    }
                })
            });
    } else if stats.souls_current > stats.souls_next {
        egui::Window::new("Spend souls on upgrades")
            .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
            .show(contexts.ctx_mut(), |ui| {
                ui.vertical_centered(|ui| {
                    if stats.upgrade_circle < GameStats::MAX_CIRCLES
                        && ui.button("New summoning circle").clicked()
                    {
                        stats.souls_current -= stats.souls_next;
                        stats.souls_next += stats.souls_next / 2;
                        stats.upgrade_circle += 1;
                        commands.spawn((LevelLocal, Spawner::default()));
                    }
                    if ui.button("Faster summoning").clicked() {
                        stats.souls_current -= stats.souls_next;
                        stats.souls_next += stats.souls_next / 2;
                        stats.upgrade_speed += 1;
                    }
                    if stats.upgrade_level + 1 < cardinality::<UnitPrefab>() as u8
                        && ui.button("New type of demon").clicked()
                    {
                        stats.souls_current -= stats.souls_next;
                        stats.souls_next += stats.souls_next / 2;
                        stats.upgrade_level += 1;
                    }
                    if ui.button("Appease the archdemon").clicked() {
                        stats.souls_current -= stats.souls_next;
                        stats.souls_next += stats.souls_next / 2;
                        stats.upgrade_appease += 1;
                    }
                });
            });
    }
}

fn setup_ui(mut contexts: EguiContexts, mut set: ResMut<EguiSettings>) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "Macondo".to_owned(),
        egui::FontData::from_static(include_bytes!("../assets/fonts/Macondo-Regular.ttf")),
    );
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "Macondo".to_owned());

    contexts.ctx_mut().set_fonts(fonts);
    set.scale_factor = 1.3;
}

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(not(feature = "editor"))]
        app.add_plugins(EguiPlugin);
        #[cfg(not(feature = "editor"))]
        app.add_systems(Startup, setup_ui);
        app.add_systems(
            Update,
            main_menu.in_set(Gameplay).run_if(in_state(Level::MainMenu)),
        )
        .add_systems(
            Update,
            game_ui
                .in_set(Gameplay)
                .run_if(not(in_state(Level::MainMenu))),
        );
    }
}
