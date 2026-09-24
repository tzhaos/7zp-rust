#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use p7z_core::i18n;
use p7z_platform as platform;
use p7z_ui as ui;

use gpui_kit::{component::Root, *};

fn report(error: impl std::fmt::Display) {
    let details = format!("{error:#}");
    tracing::error!(error = %details, "Application error");
    rfd::MessageDialog::new()
        .set_title("Plus7z")
        .set_description(details)
        .set_level(rfd::MessageLevel::Error)
        .show();
}

fn main() {
    let log_guard = match p7z_core::settings::directory()
        .and_then(|directory| cardo_runtime::diagnostics::init(&directory.join("logs"), "Plus7z"))
    {
        Ok(guard) => guard,
        Err(error) => {
            report(error);
            return;
        }
    };
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let language = if let Some(index) = args.iter().position(|arg| arg == "--lang") {
        if index + 1 == args.len() {
            match i18n::init() {
                Ok(()) => report(i18n::tr("language-required")),
                Err(error) => report(error),
            }
            return;
        }
        let language = args.remove(index + 1);
        args.remove(index);
        Some(language)
    } else {
        None
    };
    if let Err(error) = i18n::init().and_then(|_| p7z_core::settings::initialize()).and_then(|_| i18n::apply_preference(language.as_deref())) {
        report(error);
        return;
    }
    if matches!(
        args.first().map(String::as_str),
        Some("--register" | "--unregister" | "--prepare-install" | "--apply-update")
    ) {
        let result = if args[0] == "--apply-update" {
            platform::updater::apply()
        } else if args[0] == "--register" {
            std::env::current_exe()
                .map_err(anyhow::Error::from)
                .and_then(|path| {
                    let dll = args
                        .get(1)
                        .map(std::path::PathBuf::from)
                        .unwrap_or_else(|| path.with_file_name("p7z-explorer.dll"));
                    platform::register(&path, &dll)
                })
        } else if args[0] == "--prepare-install" {
            match args.get(1) {
                Some(directory) => platform::close_application(std::path::Path::new(directory)),
                None => Err(anyhow::anyhow!(i18n::tr("maintenance-target-required"))),
            }
        } else {
            platform::unregister()
        };
        if let Err(error) = result {
            report(error);
            drop(log_guard);
            std::process::exit(1);
        }
        return;
    }
    match platform::updater::recover() {
        Ok(Some(message)) => report(message),
        Ok(None) => {}
        Err(error) => {
            report(error);
            return;
        }
    }
    let first_path = usize::from(args.first().is_some_and(|arg| arg.starts_with("--")));
    for argument in args.iter_mut().skip(first_path) {
        match std::path::absolute(&*argument) {
            Ok(path) => *argument = path.to_string_lossy().into_owned(),
            Err(error) => {
                report(error);
                return;
            }
        }
    }
    let system = match platform::instance(&args) {
        Ok(Some(system)) => system,
        Ok(None) => return,
        Err(error) => {
            report(error);
            return;
        }
    };
    let startup = match p7z_core::settings::StartupSettings::load() {
        Ok(settings) => settings,
        Err(error) => {
            report(error);
            return;
        }
    };
    gpui_kit::application()
        .with_assets(ui::assets::AppAssets)
        .run(move |cx| {
            gpui_kit::init(cx);
            match ui::theme::load()
                .and_then(|id| ui::theme::apply(id, &startup.appearance, None, cx))
            {
                Ok(()) => {}
                Err(error) => {
                    report(error);
                    cx.quit();
                    return;
                }
            }
            cx.on_window_closed(|cx, _| {
                if cx.windows().is_empty() {
                    cx.quit();
                }
            })
            .detach();
            let bounds = WindowBounds::centered(size(px(900.), px(680.)), cx);
            cx.spawn(async move |cx| {
                let options = WindowOptions {
                    window_bounds: Some(bounds),
                    window_min_size: Some(size(px(800.), px(460.))),
                    inactive_frame_interval: None,
                    titlebar: Some(TitlebarOptions {
                        title: Some("Plus7z".into()),
                        appears_transparent: true,
                        ..Default::default()
                    }),
                    ..Default::default()
                };
                if let Err(error) = cx.open_window(options, |window, cx| {
                    let view = cx.new(|cx| ui::Workspace::new(startup, window, cx));
                    view.update(cx, |view, cx| {
                        view.attach_system(system.receiver, args, window, cx)
                    });
                    cx.new(|cx| Root::new(view, window, cx))
                }) {
                    report(error);
                    cx.update(|cx| cx.quit());
                }
            })
            .detach();
        });
}
