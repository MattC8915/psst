use std::sync::Arc;

use druid::{im::Vector, Data, Lens};
use serde::{Deserialize, Serialize};
use time::{formatting::Formattable, macros::format_description, Date};

use crate::data::{ArtistLink, Cached, Image, Promise, Track};

#[derive(Clone, Data, Lens)]
pub struct AlbumDetail {
    pub album: Promise<Cached<Arc<Album>>, AlbumLink>,
}

#[derive(Clone, Data, Lens, serde::Deserialize, serde::Serialize)]
pub struct Album {
    pub id: Arc<str>,
    pub name: Arc<str>,
    pub album_type: AlbumType,
    #[serde(default)]
    pub images: Vector<Image>,
    #[serde(default)]
    pub artists: Vector<ArtistLink>,
    #[serde(default)]
    pub copyrights: Vector<Copyright>,
    #[serde(default = "super::utils::default_str")]
    #[serde(deserialize_with = "super::utils::deserialize_null_arc_str")]
    pub label: Arc<str>,
    #[serde(default)]
    #[serde(deserialize_with = "super::utils::deserialize_first_page")]
    pub tracks: Vector<Arc<Track>>,
    #[serde(deserialize_with = "super::utils::deserialize_date_option")]
    #[serde(serialize_with = "super::utils::serialize_date_option")]
    #[data(same_fn = "PartialEq::eq")]
    pub release_date: Option<Date>,
    #[data(same_fn = "PartialEq::eq")]
    pub release_date_precision: Option<DatePrecision>,
}

impl Album {
    pub fn release(&self) -> String {
        self.release_with_format(match self.release_date_precision {
            Some(DatePrecision::Year) | None => format_description!("[year]"),
            Some(DatePrecision::Month) => format_description!("[month repr:long] [year]"),
            Some(DatePrecision::Day) => format_description!("[month repr:long] [day], [year]"),
        })
    }

    pub fn release_year(&self) -> String {
        self.release_with_format(format_description!("[year]"))
    }

    pub fn release_year_int(&self) -> usize {
        self.release_year().parse::<usize>().unwrap_or_else(|err| {
            log::error!("error parsing release year for {}: {}", self.name, err);
            usize::MAX
        })
    }

    fn release_with_format(&self, format: &(impl Formattable + ?Sized)) -> String {
        self.release_date
            .as_ref()
            .map(|date| date.format(format).expect("invalid format"))
            .unwrap_or_default()
    }

    pub fn image(&self, width: f64, height: f64) -> Option<&Image> {
        Image::at_least_of_size(&self.images, width, height)
    }

    pub fn url(&self) -> String {
        format!("https://open.spotify.com/album/{id}", id = self.id)
    }

    pub fn link(&self) -> AlbumLink {
        AlbumLink {
            id: self.id.clone(),
            name: self.name.clone(),
            images: self.images.clone(),
        }
    }

    pub fn has_explicit(&self) -> bool {
        self.tracks.iter().any(|t| t.explicit)
    }

    pub fn into_tracks_with_context(self: Arc<Self>) -> Vector<Arc<Track>> {
        let album_link = self.link();
        self.tracks
            .iter()
            .map(|track| {
                let mut track = track.as_ref().clone();
                track.album = Some(album_link.clone());
                Arc::new(track)
            })
            .collect()
    }
}

#[derive(Clone, Debug, Data, Lens, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct AlbumLink {
    pub id: Arc<str>,
    pub name: Arc<str>,
    #[serde(default)]
    pub images: Vector<Image>,
}

impl AlbumLink {
    pub fn image(&self, width: f64, height: f64) -> Option<&Image> {
        Image::at_least_of_size(&self.images, width, height)
    }
}

#[derive(Clone, Debug, Data, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AlbumType {
    Album,
    Single,
    Compilation,
    AppearsOn,
}

impl Default for AlbumType {
    fn default() -> Self {
        Self::Album
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Data, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DatePrecision {
    Year,
    Month,
    Day,
}

#[derive(Clone, Debug, Data, Lens, Deserialize, Serialize)]
pub struct Copyright {
    pub text: Arc<str>,
    #[serde(rename = "type")]
    pub kind: CopyrightType,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Data, Deserialize, Serialize)]
pub enum CopyrightType {
    #[serde(rename = "C")]
    Copyright,
    #[serde(rename = "P")]
    Performance,
}
