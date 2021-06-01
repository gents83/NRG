use std::path::{Path, PathBuf};

use nrg_resources::{get_absolute_data_path, ResourceId, ResourceTrait, SharedData, SharedDataRw};
use nrg_serialize::{generate_random_uid, generate_uid_from_string, INVALID_UID};

use crate::INVALID_INDEX;

pub type TextureId = ResourceId;

pub struct TextureInstance {
    id: ResourceId,
    path: PathBuf,
    texture_handler_index: i32,
    texture_index: i32,
    layer_index: i32,
}

impl ResourceTrait for TextureInstance {
    fn id(&self) -> ResourceId {
        self.id
    }
    fn path(&self) -> PathBuf {
        self.path.clone()
    }
}

impl Default for TextureInstance {
    fn default() -> Self {
        Self {
            id: generate_random_uid(),
            path: PathBuf::new(),
            texture_handler_index: INVALID_INDEX,
            texture_index: INVALID_INDEX,
            layer_index: INVALID_INDEX,
        }
    }
}

impl TextureInstance {
    pub fn find_id(shared_data: &SharedDataRw, texture_path: &Path) -> TextureId {
        let path = get_absolute_data_path(texture_path);
        SharedData::match_resource(shared_data, |t: &TextureInstance| t.path == path)
    }
    pub fn get_path(&self) -> &Path {
        self.path.as_path()
    }
    pub fn set_texture_data(
        &mut self,
        texture_handler_index: u32,
        texture_index: u32,
        layer_index: u32,
    ) -> &mut Self {
        self.texture_handler_index = texture_handler_index as _;
        self.texture_index = texture_index as _;
        self.layer_index = layer_index as _;
        self
    }
    pub fn get_texture_handler_index(&self) -> i32 {
        self.texture_handler_index
    }
    pub fn get_texture_index(&self) -> i32 {
        self.texture_index
    }
    pub fn get_layer_index(&self) -> i32 {
        self.layer_index
    }
    pub fn create_from_path(shared_data: &SharedDataRw, texture_path: &Path) -> TextureId {
        let path = get_absolute_data_path(texture_path);
        let texture_id =
            { SharedData::match_resource(shared_data, |t: &TextureInstance| t.path == path) };
        if texture_id != INVALID_UID {
            return texture_id;
        }
        let mut data = shared_data.write().unwrap();
        data.add_resource(TextureInstance {
            id: generate_uid_from_string(path.to_str().unwrap()),
            path,
            texture_handler_index: INVALID_INDEX,
            texture_index: INVALID_INDEX,
            layer_index: INVALID_INDEX,
        })
    }
}
