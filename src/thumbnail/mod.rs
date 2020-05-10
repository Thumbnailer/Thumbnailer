use crate::errors::ApplyError;
use crate::generic::OperationContainer;
use crate::thumbnail::data::ThumbnailData;
use crate::{
    errors::FileError, generic::GenericThumbnail, thumbnail::operations::Operation, Target,
};
use image::io::Reader;
use image::DynamicImage;
use std::path::Path;
use std::path::PathBuf;

pub mod collection;
pub mod data;
pub mod operations;
pub mod static_thumb;

pub use collection::ThumbnailCollection;
pub use collection::ThumbnailCollectionBuilder;
pub use static_thumb::StaticThumbnail;

#[derive(Debug)]
pub struct Thumbnail {
    data: ThumbnailData,
    ops: Vec<Box<dyn Operation>>,
}

impl OperationContainer for Thumbnail {
    fn add_op(&mut self, op: Box<dyn Operation>) {
        self.ops.push(op);
    }
}

impl Thumbnail {
    pub fn load(path: PathBuf) -> Result<Thumbnail, FileError> {
        Ok(Thumbnail {
            data: ThumbnailData::load(path)?,
            ops: vec![],
        })
    }

    /// This function creates and returns a new `Thumbnail` from an existing DynamicImage.
    ///
    /// # Arguments
    ///
    /// * `path_name` - A custom path for the new `Thumbnail`
    /// * `dynamic_image` - The `DynamicImage` that should be contained in the `Thumbnail`
    ///
    /// # Panic
    ///
    /// This function won't panic.
    pub fn from_dynamic_image(path_name: &str, dynamic_image: DynamicImage) -> Self {
        Thumbnail {
            data: ThumbnailData::from_dynamic_image(path_name, dynamic_image),
            ops: vec![],
        }
    }

    pub fn into_data(self) -> ThumbnailData {
        self.data
    }

    pub fn get_path(&self) -> PathBuf {
        self.data.get_path()
    }

    pub fn clone_static_copy(&mut self) -> Option<StaticThumbnail> {
        let src_path = self.data.get_path();
        match self.get_dyn_image() {
            Ok(i) => Some(StaticThumbnail::new(src_path, i.clone())),
            Err(_) => None,
        }
    }

    pub fn try_clone_and_load(&mut self) -> Result<Thumbnail, FileError> {
        let ops = self.ops.clone();
        let image = self.data.try_clone_and_load()?;
        Ok(Thumbnail { data: image, ops })
    }

    pub fn can_load(path: &Path) -> bool {
        if !path.is_file() {
            return false;
        }

        match Reader::open(path) {
            Err(_) => return false,
            Ok(reader) => match reader.format() {
                Some(_) => true,
                None => false,
            },
        }
    }

    pub(crate) fn get_dyn_image<'a>(&mut self) -> Result<&mut image::DynamicImage, FileError> {
        return self.data.get_dyn_image();
    }
}

impl GenericThumbnail for Thumbnail {
    fn apply(&mut self) -> Result<&mut dyn GenericThumbnail, ApplyError> {
        self.data.apply_ops_list(&self.ops)?;

        self.ops.clear();

        Ok(self)
    }

    fn apply_store(mut self, target: &Target) -> Result<Vec<PathBuf>, ApplyError> {
        self.apply()?;
        self.store(target)
    }

    fn apply_store_keep(&mut self, target: &Target) -> Result<Vec<PathBuf>, ApplyError> {
        self.apply()?;
        self.store_keep(target)
    }

    fn store(self, target: &Target) -> Result<Vec<PathBuf>, ApplyError> {
        match target.store(&mut self.into_data(), None) {
            Ok(files) => Ok(files),
            Err(err) => Err(ApplyError::StoreError(err)),
        }
    }

    fn store_keep(&mut self, target: &Target) -> Result<Vec<PathBuf>, ApplyError> {
        match target.store(&mut self.data, None) {
            Ok(files) => Ok(files),
            Err(err) => Err(ApplyError::StoreError(err)),
        }
    }
}
