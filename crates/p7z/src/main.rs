#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
use gpui_kit::*;
use p7z_core::i18n;
use p7z_platform as platform;
use p7z_ui as ui;
fn report(error: anyhow::Error) {
    let details = format!("{error:#}");
    tracing::error!(error = %details, "Application error");
    rfd::MessageDialog::new()
        .set_title("Plus7z")
        .set_description(details)
        .set_level(rfd::MessageLevel::Error)
        .show();
}
struct Services {
    startup: Option<p7z_core::settings::StartupSettings>,
}
impl cardo_app::AppServices for Services {
    type View = ui::Workspace;
    fn initialize(
        &mut self,
        _: &cardo_app::AppDescriptor,
        args: &mut Vec<String>,
    ) -> anyhow::Result<()> {
        let language = if let Some(index) = args.iter().position(|arg| arg == "--lang") {
            if index + 1 == args.len() {
                i18n::init()?;
                anyhow::bail!(i18n::tr("language-required"));
            }
            let language = args.remove(index + 1);
            args.remove(index);
            Some(language)
        } else {
            None
        };
        i18n::init()?;
        p7z_core::settings::initialize()?;
        i18n::apply_preference(language.as_deref())?;
        Ok(())
    }
    fn maintenance(&mut self, args: &[String]) -> Option<anyhow::Result<()>> {
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
            return Some(result);
        }
        None
    }
    fn prepare(&mut self, args: &mut Vec<String>) -> anyhow::Result<()> {
        if let Some(message) = platform::updater::recover()? {
            report(anyhow::anyhow!(message));
        }
        let first_path = usize::from(args.first().is_some_and(|arg| arg.starts_with("--")));
        for argument in args.iter_mut().skip(first_path) {
            *argument = std::path::absolute(&*argument)?
                .to_string_lossy()
                .into_owned();
        }
        self.startup = Some(p7z_core::settings::StartupSettings::load()?);
        Ok(())
    }
    fn initialize_ui(&mut self, cx: &mut App) -> anyhow::Result<()> {
        let startup = self.startup.as_ref().expect("prepared startup");
        ui::theme::apply(ui::theme::load()?, &startup.appearance, None, cx)
    }
    fn create(&mut self, window: &mut Window, cx: &mut App) -> Entity<Self::View> {
        let startup = self.startup.take().expect("prepared startup");
        cx.new(|cx| ui::Workspace::new(startup, window, cx))
    }
    fn launch(
        view: &mut Self::View,
        args: Vec<String>,
        window: &mut Window,
        cx: &mut Context<Self::View>,
    ) {
        view.receive_launch(args, window, cx);
    }
}
fn main() {
    let result = (|| {
        let descriptor = cardo_app::AppDescriptor {
            id: "p7z".into(),
            name: "Plus7z".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            data_directory: p7z_core::settings::directory()?,
            window: cardo_app::WindowSpec {
                preferred: size(px(900.), px(680.)),
                minimum: size(px(800.), px(460.)),
            },
            single_instance: true,
            update: Some(platform::updater::service()?.config),
        };
        cardo_app::run(
            descriptor,
            ui::assets::AppAssets,
            Services { startup: None },
            std::env::args().skip(1).collect(),
            report,
        )
    })();
    if let Err(error) = result {
        report(error);
        std::process::exit(1);
    }
}
