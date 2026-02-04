//! Desktop app for running homebuyer Monte Carlo simulations.

mod persistence;
mod simulation;
mod ui;

use bevy::prelude::*;
use bevy::window::WindowPlugin;
use bevy_egui::egui::{self, Ui};
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass};
use std::thread;

use persistence::{config, export};
use simulation::{AggregatedResults, SimulationEngine};
use ui::app_state::{ActiveTab, AppState, SimulationChannel, SimulationMessage};
use ui::panels::{draw_comparison_panel, draw_input_panel, draw_results_panel, draw_summary_panel};

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Homebuyer Monte Carlo Simulation".to_string(),
                        resolution: (1400, 900).into(),
                        ..default()
                    }),
                    ..default()
                })
                .set(bevy::log::LogPlugin {
                    filter: "wgpu=error,naga=warn,bevy_render=info".to_string(),
                    ..default()
                }),
        )
        .add_plugins(EguiPlugin::default())
        .insert_resource(SimulationChannel::new())
        .init_non_send_resource::<AppState>()
        .add_systems(Startup, setup_camera)
        .add_systems(Update, message_receiver_system)
        .add_systems(EguiPrimaryContextPass, ui_system)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn message_receiver_system(
    mut state: NonSendMut<AppState>,
    channel: Res<SimulationChannel>,
) {
    if let Ok(receiver) = channel.receiver.lock() {
        while let Ok(msg) = receiver.try_recv() {
            match msg {
                SimulationMessage::Progress(done, total) => {
                    state.progress = (done, total);
                    state.status_message = format!("Running... {}/{}", done, total);
                }
                SimulationMessage::Complete(results) => {
                    state.results = Some(*results);
                    state.is_running = false;
                    state.status_message = "Simulation complete".to_string();
                    state.update_comparison();
                    state.active_tab = ActiveTab::Summary;
                }
                SimulationMessage::Error(err) => {
                    state.is_running = false;
                    state.status_message = format!("Error: {}", err);
                }
            }
        }
    }
}

fn ui_system(
    mut contexts: EguiContexts,
    mut state: NonSendMut<AppState>,
    channel: Res<SimulationChannel>,
) -> Result {
    let ctx = contexts.ctx_mut()?;
    ui::apply_theme(ctx);

    // Top menu bar
    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        egui::menu::bar(ui, |ui: &mut Ui| {
            ui.menu_button("File", |ui: &mut Ui| {
                if ui.button("Save Configuration...").clicked() {
                    if let Some(result) = config::save_config_dialog(&state.config) {
                        match result {
                            Ok(()) => state.status_message = "Configuration saved".to_string(),
                            Err(e) => state.status_message = e,
                        }
                    }
                    ui.close_menu();
                }
                if ui.button("Load Configuration...").clicked() {
                    if let Some(result) = config::load_config_dialog() {
                        match result {
                            Ok(config) => {
                                state.config = config;
                                state.status_message = "Configuration loaded".to_string();
                            }
                            Err(e) => state.status_message = e,
                        }
                    }
                    ui.close_menu();
                }
                ui.separator();
                if ui
                    .add_enabled(state.results.is_some(), egui::Button::new("Export Results (JSON)..."))
                    .clicked()
                {
                    if let Some(ref results) = state.results {
                        if let Some(result) = export::export_results_json_dialog(results) {
                            match result {
                                Ok(()) => state.status_message = "Results exported to JSON".to_string(),
                                Err(e) => state.status_message = e,
                            }
                        }
                    }
                    ui.close_menu();
                }
                if ui
                    .add_enabled(state.results.is_some(), egui::Button::new("Export Results (CSV)..."))
                    .clicked()
                {
                    if let Some(ref results) = state.results {
                        if let Some(result) = export::export_results_csv_dialog(results) {
                            match result {
                                Ok(()) => state.status_message = "Results exported to CSV".to_string(),
                                Err(e) => state.status_message = e,
                            }
                        }
                    }
                    ui.close_menu();
                }
            });

            ui.menu_button("View", |ui: &mut Ui| {
                if ui.button("Setup").clicked() {
                    state.active_tab = ActiveTab::Configuration;
                    ui.close_menu();
                }
                if ui.button("Results").clicked() {
                    state.active_tab = ActiveTab::Results;
                    ui.close_menu();
                }
                if ui.button("Comparison").clicked() {
                    state.active_tab = ActiveTab::Comparison;
                    ui.close_menu();
                }
                if ui.button("Summary").clicked() {
                    state.active_tab = ActiveTab::Summary;
                    ui.close_menu();
                }
            });

            ui.menu_button("Help", |ui: &mut Ui| {
                if ui.button("About").clicked() {
                    state.status_message =
                        "Homebuyer Monte Carlo Simulation v0.1.0".to_string();
                    ui.close_menu();
                }
            });
        });
    });

    // Tab bar
    egui::TopBottomPanel::top("tab_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.selectable_value(&mut state.active_tab, ActiveTab::Configuration, "Setup");
            ui.selectable_value(&mut state.active_tab, ActiveTab::Results, "Results");
            ui.selectable_value(&mut state.active_tab, ActiveTab::Comparison, "Comparison");
            ui.selectable_value(&mut state.active_tab, ActiveTab::Summary, "Summary");

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let run_button = ui.add_enabled(
                    !state.is_running,
                    egui::Button::new("Run Simulation"),
                );

                if run_button.clicked() {
                    start_simulation(&mut state, channel.sender.clone());
                }
            });
        });
    });

    // Status bar
    egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.label(format!("Status: {}", state.status_message));

            if state.is_running {
                ui.separator();
                let progress = state.progress_percent();
                ui.add(
                    egui::ProgressBar::new(progress)
                        .desired_width(200.0)
                        .text(format!("{:.0}%", progress * 100.0)),
                );
            }

            if let Some(ref results) = state.results {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("{} simulations", results.num_simulations));
                });
            }
        });
    });

    // Main content
    egui::CentralPanel::default().show(ctx, |ui| {
        match state.active_tab {
            ActiveTab::Configuration => {
                draw_input_panel(ui, &mut state);
            }
            ActiveTab::Results => {
                draw_results_panel(ui, &mut state);
            }
            ActiveTab::Comparison => {
                draw_comparison_panel(ui, &state);
            }
            ActiveTab::Summary => {
                draw_summary_panel(ui, &state);
            }
        }
    });

    Ok(())
}

fn start_simulation(state: &mut AppState, sender: std::sync::mpsc::Sender<SimulationMessage>) {
    state.is_running = true;
    state.progress = (0, state.config.num_simulations);
    state.status_message = "Starting simulation...".to_string();

    let config = state.config.clone();

    thread::spawn(move || {
        let engine = SimulationEngine::new(config);

        let progress_sender = sender.clone();
        let results = engine.run_parallel(move |done, total| {
            let _ = progress_sender.send(SimulationMessage::Progress(done, total));
        });

        let aggregated = AggregatedResults::from_simulations(results, 100);
        let _ = sender.send(SimulationMessage::Complete(Box::new(aggregated)));
    });
}
