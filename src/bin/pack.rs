use ree_save_core::game_context::{GameData, load_game_configs};


pub fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info")
    ).init();

    let game_configs = load_game_configs("game_configs.json")?;

    for (game, config) in &game_configs {
        if let Some(bundle) = &config.bundle {
            log::info!("Packing {game:?}");
            let game_data = GameData::load(config, true);
            let game_data = match game_data {
                Ok(g) => g,
                Err(e) => {
                    log::error!("Could not load game data from config {:?}: {e}", config);
                    continue
                }
            };

            let config = bincode::config::standard();
            let game_data_bincode = bincode::encode_to_vec(&game_data.bundle, config)?;
            std::fs::write(bundle, game_data_bincode)?;
        }
    }
    Ok(())
}
