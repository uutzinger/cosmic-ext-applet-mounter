// SPDX-License-Identifier: MIT

mod app;
mod runtime_ipc;
mod theme;

use cosmic_ext_applet_mounter::{fl, i18n};

fn main() -> cosmic::iced::Result {
    let mut worker_arguments = std::env::args().skip(1);
    if worker_arguments.next().as_deref()
        == Some(cosmic_ext_applet_mounter::bisync_worker::WORKER_FLAG)
    {
        let arguments = worker_arguments.collect::<Vec<_>>();
        match cosmic_ext_applet_mounter::bisync_worker::run(&arguments) {
            Ok(message) => println!("{message}"),
            Err(error) => {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        std::process::exit(0);
    }
    let requested_languages = i18n_embed::DesktopLanguageRequester::requested_languages();
    i18n::init(&requested_languages);
    let mut arguments = std::env::args().skip(1);
    let guard = arguments.next();
    if matches!(
        guard.as_deref(),
        Some(cosmic_ext_applet_mounter::vpn::METERED_GUARD_FLAG)
            | Some(cosmic_ext_applet_mounter::vpn::OFFLINE_READINESS_GUARD_FLAG)
    ) {
        let arguments = arguments.collect::<Vec<_>>();
        let result = tokio::runtime::Runtime::new()
            .map_err(|error| error.to_string())
            .and_then(|runtime| match guard.as_deref() {
                Some(cosmic_ext_applet_mounter::vpn::METERED_GUARD_FLAG) => {
                    if arguments.is_empty() {
                        runtime.block_on(cosmic_ext_applet_mounter::vpn::run_unmetered_guard())
                    } else {
                        Err(crate::fl!(
                            "ui-main-unmetered-network-guard-does-not-accept-arguments"
                        ))
                    }
                }
                Some(cosmic_ext_applet_mounter::vpn::OFFLINE_READINESS_GUARD_FLAG) => {
                    runtime.block_on(app::run_offline_readiness_guard(&arguments))
                }
                _ => unreachable!(),
            });
        if let Err(error) = result {
            eprintln!("{error}");
            std::process::exit(1);
        }
        std::process::exit(0);
    }
    if matches!(
        guard.as_deref(),
        Some(cosmic_ext_applet_mounter::sharepoint::SERVICE_GUARD_FLAG)
            | Some(cosmic_ext_applet_mounter::sharepoint::MIRROR_GUARD_FLAG)
            | Some(cosmic_ext_applet_mounter::onedrive::MIRROR_GUARD_FLAG)
            | Some(cosmic_ext_applet_mounter::mount_guard::SERVICE_GUARD_FLAG)
    ) {
        let arguments = arguments.collect::<Vec<_>>();
        let result = tokio::runtime::Runtime::new()
            .map_err(|_| fl!("mount-guard-runtime-failed"))
            .and_then(|runtime| {
                if guard.as_deref()
                    == Some(cosmic_ext_applet_mounter::sharepoint::SERVICE_GUARD_FLAG)
                {
                    runtime.block_on(cosmic_ext_applet_mounter::sharepoint::run_service_guard(
                        &arguments,
                    ))
                } else if guard.as_deref()
                    == Some(cosmic_ext_applet_mounter::sharepoint::MIRROR_GUARD_FLAG)
                {
                    runtime.block_on(
                        cosmic_ext_applet_mounter::sharepoint::run_mirror_service_guard(&arguments),
                    )
                } else if guard.as_deref()
                    == Some(cosmic_ext_applet_mounter::onedrive::MIRROR_GUARD_FLAG)
                {
                    runtime.block_on(
                        cosmic_ext_applet_mounter::onedrive::run_mirror_service_guard(&arguments),
                    )
                } else {
                    runtime.block_on(cosmic_ext_applet_mounter::mount_guard::run_service_guard(
                        &arguments,
                    ))
                }
            });
        if let Err(error) = result {
            let notice = if guard.as_deref()
                == Some(cosmic_ext_applet_mounter::sharepoint::MIRROR_GUARD_FLAG)
            {
                fl!("teams-mirror-guard-blocked", error = error)
            } else if guard.as_deref()
                == Some(cosmic_ext_applet_mounter::onedrive::MIRROR_GUARD_FLAG)
            {
                fl!("onedrive-mirror-guard-blocked", error = error)
            } else {
                fl!("mount-guard-blocked", error = error)
            };
            eprintln!("{notice}");
            std::process::exit(1);
        }
        std::process::exit(0);
    }
    let mode = app::AppModel::launch_mode_from_args();
    if mode == app::AppLaunchMode::GeneralSettings
        && tokio::runtime::Runtime::new()
            .is_ok_and(|runtime| runtime.block_on(runtime_ipc::activate_settings()))
    {
        return Ok(());
    }
    if app::AppModel::is_standalone_mode(mode) {
        let size = match mode {
            app::AppLaunchMode::GeneralSettings => cosmic::iced::Size::new(640.0, 480.0),
            app::AppLaunchMode::ReorderConnections => cosmic::iced::Size::new(480.0, 600.0),
            _ => cosmic::iced::Size::new(880.0, 720.0),
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
