use derive_more::{Display, Error};
use game_morton::Morton;
use serde::{Deserialize, Serialize};
use std::{
    convert::{TryFrom, TryInto},
    num::TryFromIntError,
};
use tracing::instrument;

pub type GridCoordinate = u32;

#[derive(Clone, Debug, Hash, Serialize, Deserialize)]
pub struct Grid<T> {
    inner: Vec<Option<T>>,
    width: GridCoordinate,
    height: GridCoordinate,
}

impl<T> Grid<T> {
    #[instrument]
    pub fn new_default(
        width: GridCoordinate,
        height: GridCoordinate,
    ) -> Result<Self, TryFromIntError>
    where
        T: Default,
    {
        Self::new_with(width, height, |_, _| T::default())
    }

    #[instrument(skip(entry))]
    pub fn new_with<F>(
        width: GridCoordinate,
        height: GridCoordinate,
        mut entry: F,
    ) -> Result<Self, TryFromIntError>
    where
        F: FnMut(GridCoordinate, GridCoordinate) -> T,
    {
        let size = usize::try_from(width)? * usize::try_from(height)?;
        let mut inner: Vec<_> = (0..size).map(|_| None).collect();
        for x in 0..width {
            for y in 0..height {
                let encoded: usize = Morton::encode_2d(x, y).try_into()?;
                inner[encoded] = Some(entry(x, y));
            }
        }

        Ok(Grid {
            inner,
            width,
            height,
        })
    }

    #[instrument(skip(self))]
    pub fn get(&self, x: GridCoordinate, y: GridCoordinate) -> Result<&T, GridGetError> {
        let encoded = Morton::encode_2d(x, y);
        let encoded: usize = encoded
            .try_into()
            .map_err(|source| GridGetError::ConversionFailed { encoded, source })?;

        self.inner
            .get(encoded)
            .map(|inner| inner.as_ref())
            .flatten()
            .ok_or_else(|| GridGetError::OutOfBounds {
                coords: (x, y),
                encoded,
                size: self.inner.len(),
            })
    }

    #[instrument(skip(self))]
    pub fn get_mut(
        &mut self,
        x: GridCoordinate,
        y: GridCoordinate,
    ) -> Result<&mut T, GridGetError> {
        let encoded = Morton::encode_2d(x, y);
        let encoded: usize = encoded
            .try_into()
            .map_err(|source| GridGetError::ConversionFailed { encoded, source })?;

        let size = self.inner.len();
        self.inner
            .get_mut(encoded)
            .map(|inner| inner.as_mut())
            .flatten()
            .ok_or(GridGetError::OutOfBounds {
                coords: (x, y),
                encoded,
                size,
            })
    }

    pub fn iter(&self) -> impl Iterator<Item = (GridCoordinate, GridCoordinate, &T)> {
        self.inner.iter().enumerate().filter_map(|(index, item)| {
            item.as_ref().map(|item| {
                let (x, y) = Morton::decode_2d(index.try_into().unwrap());
                (x, y, item)
            })
        })
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (GridCoordinate, GridCoordinate, &mut T)> {
        self.inner
            .iter_mut()
            .enumerate()
            .filter_map(|(index, item)| {
                item.as_mut().map(|item| {
                    let (x, y) = Morton::decode_2d(index.try_into().unwrap());
                    (x, y, item)
                })
            })
    }
}

#[derive(Clone, Debug, Display, Error)]
pub enum GridGetError {
    #[display(
        fmt = "coordinates ({}, {}) out of bounds. encoded index was {}, but there are only {} elements",
        "coords.0",
        "coords.1",
        encoded,
        size
    )]
    OutOfBounds {
        coords: (GridCoordinate, GridCoordinate),
        encoded: usize,
        size: usize,
    },
    #[display(fmt = "conversion from encoded value ({}) to index failed", encoded)]
    ConversionFailed {
        #[error(source)]
        source: TryFromIntError,
        encoded: <GridCoordinate as Morton>::Encoded,
    },
}
