// SPDX-License-Identifier: MIT

mod app;
mod i18n;
mod theme;

fn main() -> cosmic::iced::Result {
    let requested_languages = i18n_embed::DesktopLanguageRequester::requested_languages();
    i18n::init(&requested_languages);
    let mode = app::AppModel::launch_mode_from_args();
    if app::AppModel::is_standalone_mode(mode) {
        let mut settings =
            cosmic::app::Settings::default().size(cosmic::iced::Size::new(880.0, 720.0));
        if let Some(theme) = theme::try_load_host_cosmic_theme() {
            settings = settings.theme(theme);
        }
        cosmic::app::run::<app::AppModel>(settings, mode)
    } else {
        cosmic::applet::run::<app::AppModel>(mode)
    }
}
