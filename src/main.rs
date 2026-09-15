// SPDX-License-Identifier: MIT

mod app;
mod i18n;
mod runtime_ipc;
mod theme;

fn main() -> cosmic::iced::Result {
    let requested_languages = i18n_embed::DesktopLanguageRequester::requested_languages();
    i18n::init(&requested_languages);
    let mode = app::AppModel::launch_mode_from_args();
    if mode == app::AppLaunchMode::GeneralSettings
        && tokio::runtime::Runtime::new()
            .is_ok_and(|runtime| runtime.block_on(runtime_ipc::activate_settings()))
    {
        return Ok(());
    }
    if app::AppModel::is_standalone_mode(mode) {
        let size = if mode == app::AppLaunchMode::GeneralSettings {
            cosmic::iced::Size::new(640.0, 480.0)
        } else {
            cosmic::iced::Size::new(880.0, 720.0)
        };
        let mut settings = cosmic::app::Settings::default().size(size);
        if let Some(theme) = theme::try_load_host_cosmic_theme() {
            settings = settings.theme(theme);
        }
        cosmic::app::run::<app::AppModel>(settings, mode)
    } else {
        cosmic::applet::run::<app::AppModel>(mode)
    }
}
