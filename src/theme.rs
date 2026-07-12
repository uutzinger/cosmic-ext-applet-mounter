// SPDX-License-Identifier: MIT

use std::path::PathBuf;
use std::sync::Arc;

use cosmic::cosmic_config::CosmicConfigEntry;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct SrgbaColor {
    red: f32,
    green: f32,
    blue: f32,
}

#[derive(Debug, Deserialize)]
struct AccentFile {
    base: SrgbaColor,
}

pub fn try_load_host_cosmic_theme() -> Option<cosmic::Theme> {
    let home = std::env::var_os("HOME").map(PathBuf::from)?;
    let cosmic_cfg = home.join(".config").join("cosmic");
    let is_dark_path = cosmic_cfg
        .join("com.system76.CosmicTheme.Mode")
        .join("v1")
        .join("is_dark");
    let is_dark_text = std::fs::read_to_string(is_dark_path).ok()?;
    let is_dark = is_dark_text.trim() == "true";

    let theme_id = if is_dark {
        cosmic::cosmic_theme::DARK_THEME_ID
    } else {
        cosmic::cosmic_theme::LIGHT_THEME_ID
    };
    let host_config_root = home.join(".config");
    let host_theme = cosmic::cosmic_config::Config::with_custom_path(
        theme_id,
        cosmic::cosmic_theme::Theme::VERSION,
        host_config_root.clone(),
    )
    .ok()
    .map(
        |config| match cosmic::cosmic_theme::Theme::get_entry(&config) {
            Ok(theme) => theme,
            Err((_errors, fallback)) => fallback,
        },
    );

    let theme_name = if is_dark {
        "CosmicTheme.Dark"
    } else {
        "CosmicTheme.Light"
    };
    let accent_path = cosmic_cfg
        .join(format!("com.system76.{theme_name}"))
        .join("v1")
        .join("accent");
    let accent = std::fs::read_to_string(&accent_path)
        .ok()
        .and_then(|value| ron::from_str::<AccentFile>(&value).ok())
        .map(|value| {
            cosmic::cosmic_theme::palette::rgb::Srgb::new(
                value.base.red,
                value.base.green,
                value.base.blue,
            )
        });

    let theme = if let Some(theme) = host_theme {
        theme
    } else {
        let builder = if is_dark {
            cosmic::cosmic_theme::ThemeBuilder::dark()
        } else {
            cosmic::cosmic_theme::ThemeBuilder::light()
        };
        if let Some(accent) = accent {
            builder.accent(accent).build()
        } else {
            builder.build()
        }
    };
    Some(cosmic::Theme::custom(Arc::new(theme)))
}
