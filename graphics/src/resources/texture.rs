use std::path::{Path, PathBuf};

use nrg_resources::{
    convert_from_local_path, FileResource, ResourceData, ResourceId, ResourceRef, SharedData,
    SharedDataRw, DATA_FOLDER,
};
use nrg_serialize::{generate_uid_from_string, INVALID_UID};

use crate::INVALID_INDEX;

pub type TextureId = ResourceId;
pub type TextureRc = ResourceRef<TextureInstance>;

pub struct TextureInstance {
    id: ResourceId,
    path: PathBuf,
    texture_index: i32,
    layer_index: i32,
    is_initialized: bool,
}

impl Default for TextureInstance {
    fn default() -> Self {
        Self {
            id: INVALID_UID,
            path: PathBuf::new(),
            texture_index: INVALID_INDEX,
            layer_index: INVALID_INDEX,
            is_initialized: false,
        }
    }
}

impl ResourceData for TextureInstance {
    fn id(&self) -> ResourceId {
        self.id
    }
}

impl FileResource for TextureInstance {
    fn path(&self) -> &Path {
        self.path.as_path()
    }
    fn create_from_file(shared_data: &SharedDataRw, filepath: &Path) -> TextureRc {
        let path = convert_from_local_path(PathBuf::from(DATA_FOLDER).as_path(), filepath);
        let texture_id =
            { SharedData::match_resource(shared_data, |t: &TextureInstance| t.path == path) };
        if texture_id != INVALID_UID {
            return SharedData::get_resource::<Self>(shared_data, texture_id);
        }
        SharedData::add_resource(shared_data, TextureInstance::create(filepath))
    }
}

impl TextureInstance {
    pub fn find_id(shared_data: &SharedDataRw, texture_path: &Path) -> TextureId {
        let path = convert_from_local_path(PathBuf::from(DATA_FOLDER).as_path(), texture_path);
        SharedData::match_resource(shared_data, |t: &TextureInstance| t.path == path)
    }
    pub fn path(&self) -> &Path {
        self.path.as_path()
    }
    pub fn set_texture_data(&mut self, texture_index: u32, layer_index: u32) -> &mut Self {
        self.texture_index = texture_index as _;
        self.layer_index = layer_index as _;
        self.is_initialized = true;
        self
    }
    pub fn invalidate(&mut self) {
        self.is_initialized = false;
        println!("Texture {:?} will be reloaded", self.path);
    }
    pub fn is_initialized(&self) -> bool {
        self.is_initialized
    }
    pub fn texture_index(&self) -> i32 {
        self.texture_index
    }
    pub fn layer_index(&self) -> i32 {
        self.layer_index
    }
    fn create(texture_path: &Path) -> TextureInstance {
        TextureInstance {
            id: generate_uid_from_string(texture_path.to_str().unwrap()),
            path: texture_path.to_path_buf(),
            ..Default::default()
        }
    }
}
