use std::{collections::{HashMap, VecDeque}, sync::{Arc, RwLock, mpsc::{Receiver, Sender, channel}}};

use eframe::{
    self,
    egui::{Align, CentralPanel, Layout, MenuBar, TopBottomPanel},
};
use egui_dock::{DockArea, DockState};
use ree_lib::language::Language;
use ree_save_core::{edit::copy::CopyBuffer, game_context::{GameConfigs, GameData, load_game_configs}, save::game::Game};


use crate::{config::{Config, load_config_checked}, tab::{SaveFileView, Tab}, viewer::Viewer};

pub struct App {
    tree: DockState<Tab>,
    config: Config,
    config_path: String,
    game_configs: GameConfigs,
    game_contexts: Arc<RwLock<HashMap<Game, GameData>>>,
    game_load_queue: VecDeque<Game>,
    loading_game: Option<Game>,
    game_data_receiver: Receiver<GameData>,
    game_data_sender: Sender<GameData>,
    copy_buffer: CopyBuffer
}

impl App {
    pub fn new(config_path: String, config: Config) -> Self {
        let dock_state = DockState::new(vec![Tab::from(SaveFileView::new(&config))]);
        let game_configs = load_game_configs("game_configs.json")
            .unwrap_or_default();

        let (tx, rx) = channel::<GameData>();
        Self {
            tree: dock_state,
            game_configs,
            config_path,
            game_contexts: Arc::new(RwLock::new(HashMap::new())),
            config,
            game_load_queue: VecDeque::new(),
            loading_game: None,
            game_data_sender: tx,
            game_data_receiver: rx,
            copy_buffer: CopyBuffer::default()
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        if let Some(game) = self.loading_game {
            if let Ok(game_data) = self.game_data_receiver.try_recv() {
                self.game_contexts.write().unwrap().insert(game, game_data);
                self.loading_game = None;
            }
        }
        TopBottomPanel::top("Menu Bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("REE Save Editor");

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.hyperlink_to("GitHub", "https://github.com/kvasszn/ree-save-editor");
                    ui.separator();
                });
            });

            MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Empty Save File").clicked() {
                        let file_view = SaveFileView::new(&self.config);
                        let surface = self.tree.main_surface_mut();
                        surface.push_to_focused_leaf(Tab::from(file_view));
                    }
                });

                // TODO: add a live update type thing that looks at file modification time from last
                // reloaded
                if ui.button("Reload Config").clicked() {
                    match load_config_checked(&self.config_path) {
                        Ok(config) => self.config = config,
                        Err(e) => log::error!("Error: {e}. Could not load config from path {}", self.config_path),
                    }
                }

                ui.menu_button("Options", |ui| {
                    ui.style_mut().wrap_mode = Some(eframe::egui::TextWrapMode::Extend);
                    ui.menu_button(self.config.language.to_string(), |ui| {
                        use strum::IntoEnumIterator;
                        for option in Language::iter().filter(|x| INGAME_LANGUAGES.contains(x)) {
                                ui.selectable_value(
                                    &mut self.config.language,
                                    option,
                                    option.to_string(),
                                );
                            }
                    });
                });
            });
        });

        CentralPanel::default()
            //.frame(egui::Frame::central_panel(style)).inner_margin(0.))
            .show(ctx, |ui| {
                let mut viewer = Viewer {
                    game_contexts: &self.game_contexts,
                    config: &self.config,
                    game_load_queue: &mut self.game_load_queue,
                    copy_buffer: &mut self.copy_buffer
                };
                DockArea::new(&mut self.tree)
                    .show_close_buttons(true)
                    .tab_context_menus(true)
                    .draggable_tabs(true)
                    .show_tab_name_on_hover(true)
                    .show_leaf_close_all_buttons(true)
                    .show_secondary_button_hint(true)
                    .secondary_button_context_menu(true)
                    .secondary_button_on_modifier(true)
                    .show_inside(ui, &mut viewer);
                });

        //log::info!("queue: {:?}", self.game_load_queue);
        //log::info!("loaded: {:?}", self.game_contexts.read().unwrap().keys());

        if self.loading_game.is_none() && let Some(game) = self.game_load_queue.pop_front() 
            && !self.game_contexts.read().unwrap().contains_key(&game) {
            if let Some(game_config) = self.game_configs.get(&game) {
                let game_config = game_config.clone();
                let tx = self.game_data_sender.clone();
                self.loading_game = Some(game);
                std::thread::spawn(move || {
                    log::info!("trying to load {:?} for {:?}", game_config, game);
                    let game_data = match GameData::load(&game_config, false) {
                        Ok(game_data) => game_data,
                        Err(e) => {
                            log::error!("{e}: Could not load game data {:?}", game_config);
                            return;
                        }
                    };

                    let Ok(()) = tx.send(game_data) else {
                        log::error!("Could not send game data"); // TODO: proper error from tx.send
                                                                 // here
                        return;
                    };
                });
            }
        }
    }
}

const INGAME_LANGUAGES: [Language; 15] = [
    Language::Japanese,
    Language::English,
    Language::French,
    Language::German,
    Language::Italian,
    Language::Spanish,
    Language::Russian,
    Language::Polish,
    Language::PortugueseBr,
    Language::Korean,
    Language::TransitionalChinese,
    Language::SimplelifiedChinese,
    Language::Arabic,
    Language::Thai,
    Language::LatinAmericanSpanish,
];
