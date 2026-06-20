// SPDX-License-Identifier: MIT

mod app;
mod i18n;

fn main() -> cosmic::iced::Result {
    let requested_languages = i18n_embed::DesktopLanguageRequester::requested_languages();
    i18n::init(&requested_languages);
    let mode = app::AppModel::launch_mode_from_args();
    if app::AppModel::is_standalone_mode(mode) {
        let settings = cosmic::app::Settings::default().size(cosmic::iced::Size::new(880.0, 720.0));
        cosmic::app::run::<app::AppModel>(settings, mode)
    } else {
        cosmic::applet::run::<app::AppModel>(mode)
    }
}
