use figment::providers::{Format, Serialized, Toml};
use figment::{Figment, Profile};
use rocket::Config as RocketConfig;
use serde::{Deserialize, Serialize};
use std::env::var;

pub fn get_figment_for_rocket() -> Figment {
    Figment::from(rocket::Config::figment())
        .merge(Serialized::defaults(RocketConfig::default()))
        .merge(Toml::file("exercise_rocket.toml").nested())
        .merge(
            Toml::file(
                var("EXERCISE_ROCKET_CONFIG_PATH")
                    .unwrap_or("exercise_rocket.local.toml".to_string()),
            )
            .nested(),
        )
        .select(Profile::from_env_or("EXERCISE_ROCKET_PROFILE", "default"))
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub sqlite_path: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            sqlite_path: "./sqlite.db".to_string(),
        }
    }
}

pub fn get_figment_for_other() -> Figment {
    Figment::new()
        .merge(Serialized::defaults(Config::default()))
        .merge(Toml::file("exercise.toml").nested())
        .merge(Toml::file("exercise.local.toml").nested())
        .merge(
            Toml::file(var("EXERCISE_CONFIG_PATH").unwrap_or("exercise.local.toml".to_string()))
                .nested(),
        )
        .select(Profile::from_env_or("EXERCISE_PROFILE", "default"))
}
