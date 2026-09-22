use super::{
    Cancellation, CreateOptions, Engine, Format, Level, Output, Threads, Volume,
    engine::{input_list, list_argument, publish},
};
use anyhow::{Context, Result, bail};
use cardo_7zp_commands::{ArchiveFormat, HashMethod, archive_name};
use cardo_7zp_core::i18n::tr;
use std::path::PathBuf;
use std::sync::atomic::Ordering;

impl Engine {
    pub fn quick_compress(
        &self,
        sources: &[PathBuf],
        format: ArchiveFormat,
        password: &str,
        cancel: &Cancellation,
    ) -> Result<PathBuf> {
        let name = archive_name(sources, sources[0].is_dir(), false);
        let destination = sources[0]
            .parent()
            .context(tr("destination-required"))?
            .join(format!("{name}.{}", format.value()));
        let format = Format::from_command(format);
        self.create(
            sources,
            &destination,
            &CreateOptions {
                format,
                level: Level::Normal,
                method: format.default_method(),
                threads: Threads::Auto,
                volume: Volume::None,
                solid: true,
                password: password.into(),
                encrypt_names: false,
            },
            cancel,
        )
    }

    pub fn checksum(
        &self,
        sources: &[PathBuf],
        method: HashMethod,
        cancel: &Cancellation,
    ) -> Result<Output> {
        let inputs = input_list(sources)?;
        self.run(
            vec![
                "h".into(),
                format!("-scrc{}", method.value()).into(),
                "-sccUTF-8".into(),
                "-scsUTF-8".into(),
                "-spd".into(),
                list_argument("-i@", inputs.as_ref()),
            ],
            cancel,
        )
    }

    pub fn generate_checksum(&self, sources: &[PathBuf], cancel: &Cancellation) -> Result<PathBuf> {
        let name = archive_name(sources, sources[0].is_dir(), true);
        let parent = sources[0].parent().context(tr("destination-required"))?;
        let destination = parent.join(format!("{name}.sha256"));
        let stage = tempfile::tempdir_in(parent)?;
        let temporary = stage.path().join(destination.file_name().unwrap());
        let inputs = input_list(sources)?;
        let output = self.run_in(
            vec![
                "a".into(),
                "-thash".into(),
                "-scrcSHA256".into(),
                "-sccUTF-8".into(),
                "-scsUTF-8".into(),
                "-spd".into(),
                "--".into(),
                temporary.as_os_str().into(),
                list_argument("@", inputs.as_ref()),
            ],
            Some(parent),
            cancel,
            None,
        )?;
        if output.warning {
            bail!(tr("checksum-warning"));
        }
        publish(&temporary, &destination, false).context(tr("publish-failed"))?;
        Ok(destination)
    }

    pub fn verify_checksums(&self, sources: &[PathBuf], cancel: &Cancellation) -> Result<Output> {
        let mut result = Output {
            text: String::new(),
            warning: false,
        };
        for path in sources {
            let output = self.run_in(
                vec![
                    "t".into(),
                    "-thash".into(),
                    "-sccUTF-8".into(),
                    "--".into(),
                    path.as_os_str().into(),
                ],
                path.parent(),
                cancel,
                None,
            );
            match output {
                Ok(output) => {
                    result.text.push_str(&output.text);
                    result.warning |= output.warning;
                }
                Err(error) => {
                    if cancel.load(Ordering::Relaxed) {
                        return Err(error);
                    }
                    result
                        .text
                        .push_str(&format!("{}\n{error:#}\n", path.display()));
                    result.warning = true;
                }
            }
        }
        Ok(result)
    }
}
