use bevy_asset::RenderAssetUsages;
use bevy_image::{Image, TextureFormatPixelInfo};
use bevy_render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bytemuck::cast_slice;
use image::{DynamicImage, ImageBuffer};

/// Converts a [`DynamicImage`] to an [`Image`].
pub fn from_dynamic(dyn_img: DynamicImage, is_srgb: bool) -> Image {
    let width;
    let height;

    let data: Vec<u8>;
    let format: TextureFormat;

    match dyn_img {
        DynamicImage::ImageLuma8(i) => {
            let i = DynamicImage::ImageLuma8(i).into_rgba8();
            width = i.width();
            height = i.height();
            format = if is_srgb {
                TextureFormat::Rgba8UnormSrgb
            } else {
                TextureFormat::Rgba8Unorm
            };

            data = i.into_raw();
        }
        DynamicImage::ImageLumaA8(i) => {
            let i = DynamicImage::ImageLumaA8(i).into_rgba8();
            width = i.width();
            height = i.height();
            format = if is_srgb {
                TextureFormat::Rgba8UnormSrgb
            } else {
                TextureFormat::Rgba8Unorm
            };

            data = i.into_raw();
        }
        DynamicImage::ImageRgb8(i) => {
            let i = DynamicImage::ImageRgb8(i).into_rgba8();
            width = i.width();
            height = i.height();
            format = if is_srgb {
                TextureFormat::Rgba8UnormSrgb
            } else {
                TextureFormat::Rgba8Unorm
            };

            data = i.into_raw();
        }
        DynamicImage::ImageRgba8(i) => {
            width = i.width();
            height = i.height();
            format = if is_srgb {
                TextureFormat::Rgba8UnormSrgb
            } else {
                TextureFormat::Rgba8Unorm
            };

            data = i.into_raw();
        }
        DynamicImage::ImageLuma16(i) => {
            width = i.width();
            height = i.height();
            format = TextureFormat::R16Uint;

            let raw_data = i.into_raw();

            data = cast_slice(&raw_data).to_owned();
        }
        DynamicImage::ImageLumaA16(i) => {
            width = i.width();
            height = i.height();
            format = TextureFormat::Rg16Uint;

            let raw_data = i.into_raw();

            data = cast_slice(&raw_data).to_owned();
        }
        DynamicImage::ImageRgb16(image) => {
            width = image.width();
            height = image.height();
            format = TextureFormat::Rgba16Uint;

            let mut local_data = Vec::with_capacity(
                width as usize * height as usize * format.pixel_size().unwrap_or_default(),
            );

            for pixel in image.into_raw().chunks_exact(3) {
                let r = pixel[0];
                let g = pixel[1];
                let b = pixel[2];
                let a = u16::MAX;

                local_data.extend_from_slice(&r.to_ne_bytes());
                local_data.extend_from_slice(&g.to_ne_bytes());
                local_data.extend_from_slice(&b.to_ne_bytes());
                local_data.extend_from_slice(&a.to_ne_bytes());
            }

            data = local_data;
        }
        DynamicImage::ImageRgba16(i) => {
            width = i.width();
            height = i.height();
            format = TextureFormat::Rgba16Uint;

            let raw_data = i.into_raw();

            data = cast_slice(&raw_data).to_owned();
        }
        DynamicImage::ImageRgb32F(image) => {
            width = image.width();
            height = image.height();
            format = TextureFormat::Rgba32Float;

            let mut local_data = Vec::with_capacity(
                width as usize * height as usize * format.pixel_size().unwrap_or_default(),
            );

            for pixel in image.into_raw().chunks_exact(3) {
                let r = pixel[0];
                let g = pixel[1];
                let b = pixel[2];
                let a = u16::MAX;

                local_data.extend_from_slice(&r.to_ne_bytes());
                local_data.extend_from_slice(&g.to_ne_bytes());
                local_data.extend_from_slice(&b.to_ne_bytes());
                local_data.extend_from_slice(&a.to_ne_bytes());
            }

            data = local_data;
        }
        DynamicImage::ImageRgba32F(image) => {
            width = image.width();
            height = image.height();
            format = TextureFormat::Rgba32Float;

            let raw_data = image.into_raw();

            data = cast_slice(&raw_data).to_owned();
        }
        // DynamicImage is now non exhaustive, catch future variants and convert them
        _ => {
            let image = dyn_img.into_rgba8();
            width = image.width();
            height = image.height();
            format = TextureFormat::Rgba8UnormSrgb;

            data = image.into_raw();
        }
    }

    Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        format,
        RenderAssetUsages::default(),
    )
}

pub fn try_into_dynamic(image: &Image) -> Option<(DynamicImage, bool)> {
    let (image, is_srgb) = match image.texture_descriptor.format {
        TextureFormat::R8Unorm => (
            DynamicImage::ImageLuma8(ImageBuffer::from_raw(
                image.texture_descriptor.size.width,
                image.texture_descriptor.size.height,
                image.data.as_ref()?.clone(),
            )?),
            false,
        ),
        TextureFormat::Rg8Unorm => (
            DynamicImage::ImageLumaA8(ImageBuffer::from_raw(
                image.texture_descriptor.size.width,
                image.texture_descriptor.size.height,
                image.data.as_ref()?.clone(),
            )?),
            false,
        ),
        TextureFormat::Rgba8UnormSrgb => (
            DynamicImage::ImageRgba8(ImageBuffer::from_raw(
                image.texture_descriptor.size.width,
                image.texture_descriptor.size.height,
                image.data.as_ref()?.clone(),
            )?),
            true,
        ),
        _ => return None,
    };
    Some((image, is_srgb))
}
