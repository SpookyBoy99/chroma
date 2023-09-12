use crate::routes::appdata::WebData;
use crate::routes::authorization::Authorization;
use crate::routes::error::{Error, WebResult};
use crate::routes::v1::PhotoQuality;
use actix_multiresponse::Payload;
use actix_web::web;
use dal::database::Photo;
use image::io::Reader;
use image::{DynamicImage, ImageOutputFormat};
use proto::GetPhotoResponse;
use serde::Deserialize;
use std::io::Cursor;
use tap::TapFallible;
use tracing::warn;

#[derive(Debug, Deserialize)]
pub struct Query {
    /// The ID of the photo to retrieve
    id: String,
    /// A preference for the quality of a photo.
    /// If the requested quality does not exist, the photo's original resolution will be returned.
    #[serde(default)]
    quality_preference: PhotoQuality,
    /// The format of the image.
    /// E.g., WebP or PNG
    #[serde(default)]
    format: ImageFormat,
}

#[derive(Debug, Default, Deserialize)]
pub enum ImageFormat {
    #[default]
    Png,
    Jpeg,
    WebP,
}

/// Retrieve a photo by its ID.
///
/// # Errors
///
/// - If the photo does not exist
/// - If something went wrong
pub async fn get(
    _: Authorization,
    data: WebData,
    query: web::Query<Query>,
) -> WebResult<Payload<GetPhotoResponse>> {
    let photo = Photo::get_by_id(&data.db, &query.id)
        .await?
        .ok_or(Error::NotFound)?;

    let photo_bytes = data
        .storage
        .get_photo_by_id(&photo.id, query.quality_preference.clone().into())
        .await?;

    let converted = convert_format(photo_bytes, &query.format)?;

    Ok(Payload(GetPhotoResponse {
        photo: Some(proto::Photo {
            id: photo.id,
            album_id: photo.album_id,
            created_at: photo.created_at,
            photo_data: converted,
        }),
    }))
}

fn convert_format(bytes: Vec<u8>, format: &ImageFormat) -> WebResult<Vec<u8>> {
    match format {
        ImageFormat::WebP => Ok(bytes),
        ImageFormat::Png => {
            let byte_count = bytes.len();
            reencode_dynamic_image(decode_image(bytes)?, ImageOutputFormat::Png, byte_count)
        }
        ImageFormat::Jpeg => {
            let byte_count = bytes.len();
            reencode_dynamic_image(
                decode_image(bytes)?,
                ImageOutputFormat::Jpeg(100),
                byte_count,
            )
        }
    }
}

fn reencode_dynamic_image(
    image: DynamicImage,
    format: ImageOutputFormat,
    byte_count: usize,
) -> WebResult<Vec<u8>> {
    let mut cursor = Cursor::new(Vec::with_capacity(byte_count));
    image
        .write_to(&mut cursor, format)
        .tap_err(|e| warn!("Failed to write image in format: {e}"))
        .map_err(|_| Error::ImageEncoding)?;

    Ok(cursor.into_inner())
}

fn decode_image(bytes: Vec<u8>) -> WebResult<DynamicImage> {
    let cursor = Cursor::new(bytes);
    let reader = Reader::with_format(cursor, image::ImageFormat::WebP);

    reader.decode()
        .tap_err(|e| warn!("Failed to decode image: {e}"))
        .map_err(|_| Error::ImageEncoding)
}
