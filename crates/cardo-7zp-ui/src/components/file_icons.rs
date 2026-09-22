use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use cardo_7zp_platform::file_icons::{self, ICON_SIZE, IconSource};
use std::{path::Path, sync::Arc};

struct SystemFileIcon;

impl Asset for SystemFileIcon {
    type Source = IconSource;
    type Output = Result<Arc<RenderImage>, ImageCacheError>;

    fn load(
        source: Self::Source,
        _: &mut App,
    ) -> impl Future<Output = Self::Output> + Send + 'static {
        async move {
            let pixels = file_icons::load(&source).map_err(ImageCacheError::from)?;
            let raster = image::RgbaImage::from_fn(ICON_SIZE, ICON_SIZE, |x, y| {
                let offset = ((y * ICON_SIZE + x) * 4) as usize;
                image::Rgba([
                    pixels[offset],
                    pixels[offset + 1],
                    pixels[offset + 2],
                    pixels[offset + 3],
                ])
            });
            Ok(Arc::new(RenderImage::new(vec![image::Frame::new(raster)])))
        }
    }
}

pub fn file_icon(path: &Path, directory: bool, local: bool, cx: &App) -> impl IntoElement + use<> {
    let source = if local {
        IconSource::Path(path.to_path_buf())
    } else if directory {
        IconSource::Directory
    } else {
        IconSource::FileType(
            path.extension()
                .unwrap_or_default()
                .to_string_lossy()
                .to_lowercase(),
        )
    };
    let extension = (!directory)
        .then(|| {
            path.extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| {
                    ext.chars()
                        .take(4)
                        .collect::<String>()
                        .to_ascii_uppercase()
                })
                .filter(|ext| !ext.is_empty())
        })
        .flatten();
    let p = crate::theme::palette(cx);
    div()
        .relative()
        .size(px(20.))
        .flex_shrink_0()
        .child(
            img(move |window: &mut Window, cx: &mut App| {
                window.use_asset::<AssetLogger<SystemFileIcon>>(&source, cx)
            })
            .size(px(20.))
            .flex_shrink_0(),
        )
        .when_some(extension, |el, extension| {
            el.child(
                div()
                    .absolute()
                    .bottom(px(-1.))
                    .right(px(-1.))
                    .h(px(10.))
                    .px(px(2.))
                    .flex()
                    .items_center()
                    .whitespace_nowrap()
                    .rounded(px(2.))
                    .bg(rgb(p.surface))
                    .border_1()
                    .border_color(rgb(p.border))
                    .text_size(px(8.))
                    .line_height(px(9.))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(rgb(p.muted))
                    .child(extension),
            )
        })
}
