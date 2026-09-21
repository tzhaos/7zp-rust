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

pub fn file_icon(path: &Path, directory: bool, local: bool) -> impl IntoElement + use<> {
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
    img(move |window: &mut Window, cx: &mut App| {
        window.use_asset::<AssetLogger<SystemFileIcon>>(&source, cx)
    })
    .size(px(20.))
    .flex_shrink_0()
}
