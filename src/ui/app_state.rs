use crate::simulation::{AggregatedResults, RentComparison, SimulationConfig};
use bevy::prelude::*;
use std::sync::mpsc::{Receiver, Sender};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ActiveTab {
    #[default]
    Configuration,
    Results,
    Comparison,
    Summary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ChartType {
    #[default]
    EffectiveMonthlyCost,
    EquityHistogram,
    EquityTrajectory,
    EffectiveCostTrajectory,
    AllEquityTrajectories,
}

#[derive(Debug, Clone)]
pub enum SimulationMessage {
    Progress(usize, usize),
    Complete(Box<AggregatedResults>),
    Error(String),
}

// Use a wrapper for thread-unsafe channel parts
pub struct AppState {
    pub config: SimulationConfig,
    pub results: Option<AggregatedResults>,
    pub comparison: Option<RentComparison>,
    pub active_tab: ActiveTab,
    pub chart_type: ChartType,
    pub is_running: bool,
    pub progress: (usize, usize),
    pub status_message: String,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            config: SimulationConfig::default(),
            results: None,
            comparison: None,
            active_tab: ActiveTab::Configuration,
            chart_type: ChartType::EffectiveMonthlyCost,
            is_running: false,
            progress: (0, 0),
            status_message: "Ready".to_string(),
        }
    }
}

impl AppState {
    pub fn progress_percent(&self) -> f32 {
        if self.progress.1 == 0 {
            0.0
        } else {
            self.progress.0 as f32 / self.progress.1 as f32
        }
    }

    pub fn update_comparison(&mut self) {
        if let Some(ref results) = self.results {
            self.comparison = Some(RentComparison::calculate(
                results,
                self.config.base.rent_equivalent,
            ));
        }
    }
}

// Channel wrapper that's Send + Sync
#[derive(Resource)]
pub struct SimulationChannel {
    pub sender: Sender<SimulationMessage>,
    pub receiver: std::sync::Mutex<Receiver<SimulationMessage>>,
}

impl SimulationChannel {
    pub fn new() -> Self {
        let (tx, rx) = std::sync::mpsc::channel();
        Self {
            sender: tx,
            receiver: std::sync::Mutex::new(rx),
        }
    }
}
