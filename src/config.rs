use figment::providers::{Format, Serialized, Toml};
use figment::{Figment, Profile};
use rocket::Config;
use std::env::var;

pub fn get_figment_for_rocket() -> Figment {
    Figment::from(rocket::Config::figment())
        .merge(Serialized::defaults(Config::default()))
        .merge(Toml::file("exercise_rocket.toml").nested())
        .merge(
            Toml::file(
                var("EXERCISE_CONFIG_PATH").unwrap_or("exercise_rocket.local.toml".to_string()),
            )
            .nested(),
        )
        .select(Profile::from_env_or("EXERCISE_PROFILE", "default"))
}
