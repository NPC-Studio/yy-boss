use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use yy_typings::{FilesystemPath, ViewPath, ViewPathLocation, YypFolder, YypResource};

use crate::{
    dirty_handler::{DirtyDrain, DirtyHandler},
    PathStrExt, Resource, ViewPathLocationExt, YyResource,
};

static ROOT_FOLDER_VIEW_PATH: once_cell::sync::Lazy<ViewPathLocation> =
    once_cell::sync::Lazy::new(ViewPathLocation::root_folder);

static ROOT_FILE_VIEW_PATH: once_cell::sync::Lazy<std::sync::RwLock<ViewPathLocation>> =
    once_cell::sync::Lazy::new(|| std::sync::RwLock::new(Default::default()));

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Vfs {
    pub resource_names: ResourceNames,
    root: FolderGraph,
    dirty_handler: DirtyHandler<ViewPathLocation>,
}

impl Vfs {
    pub(crate) fn new(yyp_name: &str) -> Self {
        *ROOT_FILE_VIEW_PATH.write().unwrap() = ViewPathLocation::root_file(yyp_name);

        Vfs {
            root: FolderGraph::root(),
            resource_names: ResourceNames::new(),
            dirty_handler: DirtyHandler::new(),
        }
    }

    pub(crate) fn load_in_folders(&mut self, folders: &[YypFolder]) {
        for new_folder in folders.iter() {
            let mut folder_graph = &mut self.root;

            // ensure subfolders are loaded in...
            for section in new_folder.folder_path.component_paths() {
                let path_to_parent = folder_graph.view_path_location();
                let section = section.trim_yy().to_owned();

                // find or insert the new folder...
                if folder_graph.folders.iter().any(|f| f.name == section) == false {
                    folder_graph.folders.push(FolderGraph {
                        name: section.clone(),
                        path_to_parent: Some(path_to_parent),
                        // all of these are defaults..below we add in specs for each
                        folders: vec![],
                        files: Files::new(),
                    });
                }

                folder_graph = folder_graph
                    .folders
                    .iter_mut()
                    .find(|f| f.name == section)
                    .unwrap();
            }

            // get the folder and add in its order and what not...
            Vfs::get_folder_mut(&mut self.root, &new_folder.folder_path).unwrap();
        }
    }

    pub(crate) fn load_in_file<T: YyResource>(&mut self, yy: &T) -> Result<(), FolderGraphError> {
        // Add to the folder graph
        let folder =
            Vfs::get_folder_mut(&mut self.root, &yy.parent_view_path().path).ok_or_else(|| {
                FolderGraphError::PathNotFound {
                    path: yy.parent_view_path().path.inner().to_string(),
                }
            })?;

        // add and sort
        folder.files.load_in(yy, &mut self.resource_names);

        Ok(())
    }

    pub(crate) fn get_folder_mut<'a>(
        root: &'a mut FolderGraph,
        view_path: &ViewPathLocation,
    ) -> Option<&'a mut FolderGraph> {
        if Self::is_root(view_path) {
            Some(root)
        } else {
            let mut folder = root;
            let mut used_root = true;

            for path in view_path.component_paths() {
                used_root = false;
                let path = path.trim_yy();
                folder = folder.folders.iter_mut().find(|f| f.name == path)?;
            }

            if used_root == false {
                Some(folder)
            } else {
                None
            }
        }
    }

    #[allow(dead_code)]
    pub(crate) fn get_folder_by_fname_mut(&mut self, name: &str) -> Option<&mut FolderGraph> {
        self.root.get_folder_by_fname_mut(name)
    }

    pub fn root_file_viewpath(&self) -> ViewPathLocation {
        ROOT_FILE_VIEW_PATH.read().unwrap().clone()
    }

    fn get_folder_inner<'a>(
        root: &'a FolderGraph,
        view_path: &ViewPathLocation,
    ) -> Option<&'a FolderGraph> {
        if Self::is_root(view_path) {
            Some(root)
        } else {
            let mut folder = root;
            let mut used_root = true;

            for path in view_path.component_paths() {
                used_root = false;
                let path = path.trim_yy();
                folder = folder.folders.iter().find(|f| f.name == path)?;
            }

            if used_root == false {
                Some(folder)
            } else {
                None
            }
        }
    }

    /// Gets a folder by the given ViewPathLocation.
    /// If a folder does not exist, or if the path points to a file, None will be returned.
    pub fn get_folder<'a>(&'a self, view_path: &ViewPathLocation) -> Option<&'a FolderGraph> {
        Self::get_folder_inner(&self.root, view_path)
    }

    /// Gets the root folder.
    pub fn get_root_folder(&self) -> &FolderGraph {
        &self.root
    }

    /// Finds the containing folder for a given file. Returns an error is no file of that name
    /// could be found.
    pub fn get_folder_by_fname(&self, name: &str) -> Result<&FolderGraph, FolderGraphError> {
        self.root
            .get_folder_by_fname(name)
            .ok_or_else(|| FolderGraphError::PathNotFound {
                path: name.to_string(),
            })
    }

    /// The root path for a folder at the root of a project.
    ///
    /// Gms2 projects have historically had immutable folders at the top of a project's
    /// virtual file system, such as "Sprites", "Objects", etc, for each resource type.
    /// In Gms2.3, that restriction has been lifted, along with the internal changes to the
    /// Yyp, so it is now possible for any folder to be at the root of the project.
    pub fn root_folder() -> &'static ViewPathLocation {
        &ROOT_FOLDER_VIEW_PATH
    }

    pub fn is_root(view_path: &ViewPathLocation) -> bool {
        *view_path == *ROOT_FOLDER_VIEW_PATH
            || *view_path == *ROOT_FILE_VIEW_PATH.read().unwrap()
            || *view_path == ViewPathLocation::default()
    }

    /// If the Path is valid, returns the type of resource on the Path. If the path is invalid,
    /// it will return None.
    pub fn path_kind(&self, vp: &ViewPath) -> Option<Item> {
        if self.get_folder(&vp.path).is_some() {
            Some(Item::Folder)
        } else if self.resource_names.get(&vp.name).is_some() {
            Some(Item::Resource)
        } else {
            None
        }
    }

    /// Adds a subfolder to the folder given at `parent_path` with the order set to the end. If a tree looks like:
    ///
    ///```txt
    /// Sprites/
    ///     - spr_player
    ///     - spr_enemy
    /// ```
    ///
    /// and user adds a folder with name `Items` to the `Sprites` folder, then the output tree will be:
    ///
    /// ```txt
    /// Sprites/
    ///     - spr_player
    ///     - spr_enemy
    ///     - Items/
    ///```
    ///
    /// `add_folder_to_end` returns a `Result<ViewPath>`, where `ViewPath` is of the newly created folder.
    /// This allows for easy sequential operations, such as adding a folder and then adding a file to that folder.
    pub fn new_folder_end<S: AsRef<str>>(
        &mut self,
        parent_path: &ViewPathLocation,
        name: S,
    ) -> Result<ViewPath, FolderGraphError> {
        let subfolder = Self::get_folder_mut(&mut self.root, parent_path).ok_or_else(|| {
            FolderGraphError::PathNotFound {
                path: parent_path.inner().to_string(),
            }
        })?;

        // Don't add a new folder with the same name...
        if subfolder.folders.iter().any(|f| f.name == name.as_ref()) {
            return Err(FolderGraphError::FolderAlreadyPresent);
        }

        // Create our Path...
        let path = parent_path.join(name.as_ref());
        subfolder.folders.push(FolderGraph::new(
            name.as_ref().to_owned(),
            subfolder.view_path_location(),
        ));

        // reserialize it
        self.dirty_handler.add(path.clone());

        Ok(ViewPath {
            path,
            name: name.as_ref().to_owned(),
        })
    }

    /// Removes an empty folder from the virtual file system. If *anything* is within this folder, it will not be deleted,
    /// including other empty folders.
    pub fn remove_empty_folder(
        &mut self,
        folder_path: &ViewPathLocation,
    ) -> Result<(), FolderGraphError> {
        let original_folder =
            Self::get_folder_mut(&mut self.root, folder_path).ok_or_else(|| {
                FolderGraphError::PathNotFound {
                    path: folder_path.inner().to_string(),
                }
            })?;

        if original_folder.files.is_empty() == false || original_folder.folders.is_empty() == false
        {
            return Err(FolderGraphError::CannotRemoveFolder);
        }

        let name = original_folder.name.clone();
        if let Some(parent_path) = original_folder.path_to_parent.clone() {
            let parent = Self::get_folder_mut(&mut self.root, &parent_path)
                .ok_or(FolderGraphError::InternalError)?;

            let pos = parent.folders.iter().position(|v| v.name == name).unwrap();
            parent.folders.remove(pos);

            // mark the remove as dirty...
            self.dirty_handler.remove(folder_path);

            Ok(())
        } else {
            Err(FolderGraphError::CannotEditRootFolder)
        }
    }

    /// Checks if a folder *can* be named a certain value at a certain path.
    ///
    /// Users don't need this function, but it is provided to simplify users lives, and to
    /// future proof the library.
    pub fn can_name_folder(
        &self,
        parent_path: &ViewPathLocation,
        new_name: &str,
    ) -> Result<(), FolderGraphError> {
        if let Some(v) = self.get_folder(parent_path) {
            if v.folders.iter().any(|v| v.name == *new_name) {
                Err(FolderGraphError::FolderAlreadyPresent)
            } else {
                Ok(())
            }
        } else {
            Err(FolderGraphError::PathNotFound {
                path: parent_path.inner().to_string(),
            })
        }
    }

    /// Renames a folder in the Vfs.
    pub fn rename_folder(
        &mut self,
        folder_path: &ViewPathLocation,
        new_name: String,
    ) -> Result<(), FolderGraphError> {
        // confirm that we *can* name the folder that...
        let parent_folder = self
            .get_folder(folder_path)
            .ok_or_else(|| FolderGraphError::PathNotFound {
                path: folder_path.inner().to_string(),
            })?
            .path_to_parent
            .clone()
            .ok_or(FolderGraphError::CannotEditRootFolder)?;

        self.can_name_folder(&parent_folder, &new_name)?;

        let original_folder =
            Self::get_folder_mut(&mut self.root, folder_path).ok_or_else(|| {
                FolderGraphError::PathNotFound {
                    path: folder_path.inner().to_string(),
                }
            })?;

        original_folder.name = new_name;
        let new_path = original_folder.view_path_location();

        self.dirty_handler.remove(folder_path);
        self.dirty_handler.add(new_path.clone());

        // fix the children...
        for children in &mut original_folder.folders {
            children.path_to_parent = Some(new_path.clone());
        }

        Ok(())
    }

    /// Renames a resource in the Vfs.
    pub(crate) fn rename_resource(
        &mut self,
        name: &str,
        resource: Resource,
        new_name: String,
    ) -> Result<(), FolderGraphError> {
        let v = self
            .resource_names
            .get_checked(name, resource)
            .map_err(FolderGraphError::ResourceNameError)?;

        if let Some(folder) = Self::get_folder_mut(&mut self.root, &v.parent_location) {
            folder
                .files
                .edit_name(name, new_name, resource, &mut self.resource_names);

            Ok(())
        } else {
            Err(FolderGraphError::InternalError)
        }
    }

    pub(crate) fn remove_non_empty_folder(
        &mut self,
        folder_path: &ViewPathLocation,
    ) -> Result<HashMap<FilesystemPath, ResourceDescriptor>, FolderGraphError> {
        if Self::is_root(folder_path) {
            return Err(FolderGraphError::CannotEditRootFolder);
        }

        let original_folder =
            Self::get_folder_mut(&mut self.root, folder_path).ok_or_else(|| {
                FolderGraphError::PathNotFound {
                    path: folder_path.inner().to_string(),
                }
            })?;

        fn remove_resource(
            fg: &mut FolderGraph,
            rn: &mut ResourceNames,
            dh: &mut DirtyHandler<ViewPathLocation>,
            buffer: &mut HashMap<FilesystemPath, ResourceDescriptor>,
        ) {
            fg.files.drain_into(rn, buffer);
            for mut folder in fg.folders.drain(..) {
                remove_resource(&mut folder, rn, dh, buffer);

                // mark the remove as dirty...
                dh.remove(&folder.view_path_location());
            }
        }

        let mut buf = Default::default();
        remove_resource(
            original_folder,
            &mut self.resource_names,
            &mut self.dirty_handler,
            &mut buf,
        );

        self.remove_empty_folder(folder_path)?;

        Ok(buf)
    }

    /// Checks if a resource with a given name exists. If it does, it will return information
    /// on that resource in the form of the `CreatedResource` token, which can tell the user
    /// the type of resource.
    pub fn get_resource_type(&self, resource_name: &str) -> Option<Resource> {
        self.resource_names.get(resource_name).map(|v| v.resource)
    }

    /// Checks if a resource with a given name and of a given type exists.
    pub fn resource_exists(&self, resource_name: &str, resource: Resource) -> bool {
        self.resource_names
            .get(resource_name)
            .filter(|v| v.resource == resource)
            .is_some()
    }

    pub(crate) fn new_resource_end<T: YyResource>(
        &mut self,
        yy: &T,
    ) -> Result<(), FolderGraphError> {
        let subfolder = Self::get_folder_mut(&mut self.root, &yy.parent_view_path().path)
            .ok_or_else(|| FolderGraphError::PathNotFound {
                path: yy.parent_view_path().path.inner().to_string(),
            })?;

        subfolder.files.add(yy, &mut self.resource_names);
        Ok(())
    }

    pub(crate) fn remove_resource(
        &mut self,
        name: &str,
        resource: Resource,
    ) -> Result<(), FolderGraphError> {
        let v = self
            .resource_names
            .get_checked(name, resource)
            .map_err(FolderGraphError::ResourceNameError)?;

        if let Some(folder) = Self::get_folder_mut(&mut self.root, &v.parent_location) {
            folder.files.remove(name, &mut self.resource_names);
            Ok(())
        } else {
            Err(FolderGraphError::InternalError)
        }
    }

    pub fn move_folder(
        &mut self,
        folder_location_to_move: ViewPathLocation,
        new_parent: &ViewPathLocation,
    ) -> Result<(), FolderGraphError> {
        // chill bro
        if folder_location_to_move == *new_parent {
            return Ok(());
        }

        let folder_to_move = Self::get_folder_inner(&self.root, &folder_location_to_move)
            .ok_or_else(|| FolderGraphError::PathNotFound {
                path: folder_location_to_move.to_string(),
            })?;

        // make sure that the dest isn't inside the start...
        if Self::get_folder_inner(&self.root, new_parent).is_some() {
            return Err(FolderGraphError::InvalidMoveDestination);
        }

        // aaand make sure that the dest is a valid folder...
        if let Some(f) = Self::get_folder_inner(&self.root, new_parent) {
            if f.folders.iter().any(|f| f.name == folder_to_move.name) {
                return Err(FolderGraphError::FolderAlreadyPresent);
            }
        } else {
            return Err(FolderGraphError::PathNotFound {
                path: new_parent.to_string(),
            });
        }

        let name = folder_to_move.name.clone();
        let parent_path = folder_to_move
            .path_to_parent
            .clone()
            .ok_or(FolderGraphError::InvalidMoveDestination)?;

        // detach...
        let parent = Self::get_folder_mut(&mut self.root, &parent_path)
            .ok_or(FolderGraphError::InternalError)?;
        let pos = parent.folders.iter().position(|v| v.name == name).unwrap();
        let f = parent.folders.remove(pos);

        // attach
        let dest = Self::get_folder_mut(&mut self.root, new_parent).unwrap();
        dest.folders.push(f);

        // mark the remove as dirty...
        self.dirty_handler.edit(folder_location_to_move);

        Ok(())
    }

    pub fn move_resource(
        &mut self,
        resource_to_move: &str,
        resource: Resource,
        new_parent: &ViewPathLocation,
    ) -> Result<(), FolderGraphError> {
        // find the resource...
        let v = self
            .resource_names
            .get_checked(resource_to_move, resource)
            .map_err(FolderGraphError::ResourceNameError)?;

        if self.get_folder(new_parent).is_none() {
            return Err(FolderGraphError::PathNotFound {
                path: new_parent.to_string(),
            });
        }

        let folder = Self::get_folder_mut(&mut self.root, &v.parent_location)
            .ok_or(FolderGraphError::InternalError)?;

        let path = folder
            .files
            .detach(resource_to_move)
            .ok_or(FolderGraphError::InternalError)?;

        let dest = Self::get_folder_mut(&mut self.root, new_parent).unwrap();

        dest.files.attach(path);

        Ok(())
    }

    pub(crate) fn serialize(
        &mut self,
        yyp_folders: &mut Vec<YypFolder>,
        yyp_resources: &mut Vec<YypResource>,
    ) {
        // refry the beans...
        let DirtyDrain {
            resources_to_reserialize,
            resources_to_remove,
            associated_values: _,
        } = self.dirty_handler.drain_all();

        for (reserialize, _) in resources_to_reserialize {
            let folder_data = Self::get_folder_inner(&self.root, &reserialize)
                .expect("always internally consistent");

            let output = YypFolder {
                folder_path: reserialize.clone(),
                common_data: yy_typings::CommonData {
                    name: folder_data.name.clone(),
                    ..Default::default()
                },
            };

            if let Some(pos) = yyp_folders
                .iter()
                .position(|v| v.folder_path == reserialize)
            {
                yyp_folders[pos] = output;
            } else {
                yyp_folders.push(output);
            }
        }

        // remove the excess beans...
        for (remove_path, _) in resources_to_remove {
            let pos = yyp_folders
                .iter()
                .position(|v| v.folder_path == remove_path)
                .expect("must exist to remove it");
            yyp_folders.remove(pos);
        }

        // resource names...
        self.resource_names.serialize(yyp_resources);
    }
}

#[derive(Debug, Clone, Eq, Serialize, Deserialize, Default)]
pub struct FolderGraph {
    pub name: String,
    pub path_to_parent: Option<ViewPathLocation>,
    pub folders: Vec<FolderGraph>,
    pub files: Files,
}

impl PartialEq for FolderGraph {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl std::hash::Hash for FolderGraph {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl FolderGraph {
    pub(super) fn root() -> FolderGraph {
        FolderGraph {
            name: "folders".to_string(),
            path_to_parent: None,
            files: Files::new(),
            folders: vec![],
        }
    }

    pub fn new(name: String, parent: ViewPathLocation) -> FolderGraph {
        FolderGraph {
            name,
            path_to_parent: Some(parent),
            files: Files::new(),
            folders: vec![],
        }
    }

    pub fn view_path_location(&self) -> ViewPathLocation {
        match &self.path_to_parent {
            Some(parent_path) => parent_path.join(&self.name),
            None => ViewPathLocation::root_folder(),
        }
    }

    pub fn view_path(&self) -> ViewPath {
        let path = self.view_path_location();

        ViewPath {
            name: self.name.clone(),
            path,
        }
    }

    pub(super) fn get_folder_by_fname_mut<'a>(
        &'a mut self,
        name: &str,
    ) -> Option<&'a mut FolderGraph> {
        if self.files.contains_name(name) {
            return Some(self);
        }

        for subfolder in self.folders.iter_mut() {
            if let Some(found) = subfolder.get_folder_by_fname_mut(name) {
                return Some(found);
            }
        }

        None
    }

    pub(super) fn get_folder_by_fname<'a>(&'a self, name: &str) -> Option<&'a FolderGraph> {
        if self.files.contains_name(name) {
            return Some(self);
        }

        for subfolder in self.folders.iter() {
            if let Some(found) = subfolder.get_folder_by_fname(name) {
                return Some(found);
            }
        }

        None
    }

    pub fn to_flat(&self, resource_names: &ResourceNames) -> FlatFolderGraph {
        let view_path = self.view_path();

        FlatFolderGraph {
            path_to_parent: self.path_to_parent.clone(),
            folders: self.folders.iter().map(|v| v.view_path()).collect(),
            files: self
                .files
                .inner()
                .iter()
                .filter_map(|v| {
                    resource_names
                        .get(&v.name)
                        .map(|rd| FlatResourceDescriptor {
                            filesystem_path: v.clone(),
                            resource_descriptor: rd.clone(),
                        })
                })
                .collect(),
            view_path,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, Ord, PartialOrd, PartialEq, Serialize, Deserialize, Hash)]
pub enum Item {
    Folder,
    Resource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlatFolderGraph {
    pub view_path: ViewPath,
    pub path_to_parent: Option<ViewPathLocation>,
    pub folders: Vec<ViewPath>,
    pub files: Vec<FlatResourceDescriptor>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlatResourceDescriptor {
    pub filesystem_path: FilesystemPath,
    pub resource_descriptor: ResourceDescriptor,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum FolderGraphError {
    #[error("path {} was not found", .path)]
    PathNotFound { path: String },

    #[error("folder already existed at that location")]
    FolderAlreadyPresent,

    #[error("file already existed at that location")]
    FileAlreadyPresent,

    #[error("foldergraph is out of sync with internal Yyp -- yypboss is in undefined state")]
    InternalError,

    #[error("couldn't remove folder, not empty")]
    CannotRemoveFolder,

    #[error("cannot remove the root folder, why are you doing that don't do that come on now")]
    CannotEditRootFolder,

    #[error("cannot move folder inside itself")]
    InvalidMoveDestination,

    #[error(transparent)]
    ResourceNameError(#[from] ResourceNameError),
}

#[derive(Debug, thiserror::Error, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(tag = "resourceNameError")]
pub enum ResourceNameError {
    #[error("cannot use that resource name, as that name is being used already by a {}", .existing_resource)]
    #[serde(rename_all = "camelCase")]
    BadResourceName { existing_resource: Resource },

    #[error("no resource found of that name")]
    NoResourceByThatName,
}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct ResourceNames {
    names: HashMap<String, ResourceDescriptor>,
    dirty_handler: DirtyHandler<String>,
}

impl ResourceNames {
    pub(crate) fn new() -> Self {
        Self {
            names: HashMap::new(),
            dirty_handler: DirtyHandler::new(),
        }
    }

    pub(crate) fn load_in_resource(&mut self, name: String, resource: ResourceDescriptor) {
        self.names.insert(name, resource);
    }

    pub(crate) fn insert(
        &mut self,
        name: String,
        resource: ResourceDescriptor,
    ) -> Option<ResourceDescriptor> {
        if let Some(ret) = self.names.insert(name.clone(), resource) {
            self.dirty_handler.edit(name);
            Some(ret)
        } else {
            self.dirty_handler.add(name);
            None
        }
    }

    pub(crate) fn remove(&mut self, name: &str) -> Option<ResourceDescriptor> {
        if let Some(output) = self.names.remove(name) {
            self.dirty_handler.remove(name);
            Some(output)
        } else {
            None
        }
    }

    pub fn get(&self, name: &str) -> Option<&ResourceDescriptor> {
        self.names.get(name)
    }

    /// Returns all the currently known names and descriptors in the project.
    pub fn inner(&self) -> &HashMap<String, ResourceDescriptor> {
        &self.names
    }

    pub(crate) fn get_checked(
        &self,
        name: &str,
        r: Resource,
    ) -> Result<&ResourceDescriptor, ResourceNameError> {
        match self.get(name) {
            Some(v) => {
                if v.resource == r {
                    Ok(v)
                } else {
                    Err(ResourceNameError::BadResourceName {
                        existing_resource: r,
                    })
                }
            }
            None => Err(ResourceNameError::NoResourceByThatName),
        }
    }

    pub(crate) fn serialize(&mut self, yyp_resources: &mut Vec<YypResource>) {
        let DirtyDrain {
            resources_to_reserialize,
            resources_to_remove,
            associated_values: _,
        } = self.dirty_handler.drain_all();

        for (refried_bean, _) in resources_to_reserialize {
            let desc = &self.names[&refried_bean];

            if let Some(pos) = yyp_resources.iter().position(|v| v.id.name == refried_bean) {
                let rsc = desc.to_yyp_resource(&refried_bean);
                yyp_resources[pos].id = rsc.id;
            } else {
                yyp_resources.push(desc.to_yyp_resource(&refried_bean));
            }
        }

        for (name, _) in resources_to_remove {
            if let Some(pos) = yyp_resources.iter().position(|v| v.id.name == name) {
                yyp_resources.remove(pos);
            }
        }
    }
}

#[derive(
    Debug, PartialEq, Eq, Clone, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "camelCase")]
pub struct ResourceDescriptor {
    pub resource: Resource,
    pub parent_location: ViewPathLocation,
}

impl ResourceDescriptor {
    pub fn new(resource: Resource, view_path_location: ViewPathLocation) -> Self {
        Self {
            resource,
            parent_location: view_path_location,
        }
    }

    pub fn to_yyp_resource(&self, name: &str) -> YypResource {
        YypResource {
            id: FilesystemPath::new(self.resource.subpath_name(), name),
        }
    }
}

#[derive(Default, Debug, Eq, PartialEq, Clone, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub struct Files(Vec<FilesystemPath>);

impl Files {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_vec(fpaths: Vec<FilesystemPath>) -> Self {
        Self(fpaths)
    }

    pub fn contains_name(&self, name: &str) -> bool {
        self.0.iter().any(|f| f.name == *name)
    }

    pub fn load_in<T: YyResource>(&mut self, yy: &T, rn: &mut ResourceNames) {
        self.0.push(FilesystemPath::new(T::SUBPATH_NAME, yy.name()));

        // add to resource names...
        rn.load_in_resource(
            yy.name().to_string(),
            ResourceDescriptor::new(T::RESOURCE, yy.parent_view_path().path),
        );
    }

    pub fn add<T: YyResource>(&mut self, yy: &T, rn: &mut ResourceNames) {
        self.attach(FilesystemPath::new(T::SUBPATH_NAME, yy.name()));

        // add to resource names...
        rn.insert(
            yy.name().to_string(),
            ResourceDescriptor::new(T::RESOURCE, yy.parent_view_path().path),
        );
    }

    pub fn drain_into(
        &mut self,
        rn: &mut ResourceNames,
        buf: &mut HashMap<FilesystemPath, ResourceDescriptor>,
    ) {
        for file in self.0.drain(..) {
            let v = rn.remove(&file.name).unwrap();
            buf.insert(file, v);
        }
    }

    pub fn remove(&mut self, name: &str, rn: &mut ResourceNames) {
        self.detach(name);
        rn.remove(name);
    }

    pub fn edit_name(
        &mut self,
        name: &str,
        new_name: String,
        resource: Resource,
        rn: &mut ResourceNames,
    ) {
        // rename our own thing...
        if let Some(fpath) = self.0.iter_mut().find(|v| v.name == name) {
            *fpath = FilesystemPath::new(resource.subpath_name(), &new_name);
        }

        // remove the old name...
        if let Some(resource_desc) = rn.remove(name) {
            rn.insert(new_name, resource_desc);
        }
    }

    pub fn attach(&mut self, fsyspath: FilesystemPath) {
        self.0.push(fsyspath);
    }

    pub fn detach(&mut self, name: &str) -> Option<FilesystemPath> {
        self.0
            .iter()
            .position(|v| v.name == name)
            .map(|p| self.0.remove(p))
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn inner(&self) -> &Vec<FilesystemPath> {
        &self.0
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::dirty_handler::DirtyState;
    use maplit::hashmap;
    // use pretty_assertions::assert_eq;
    use super::YyResource;
    use std::{collections::HashSet, path::Path};
    use yy_typings::{Script, ViewPathLocation, YypFolder};

    #[test]
    fn folder_manipulations() {
        let mut fgm = Vfs::new("project");
        assert_eq!(
            *ROOT_FILE_VIEW_PATH.read().unwrap(),
            ViewPathLocation::root_file("project"),
        );

        let new_folder = fgm.new_folder_end(Vfs::root_folder(), "Sprites").unwrap();

        let root = FolderGraph {
            name: "folders".to_string(),
            path_to_parent: None,
            files: Files::new(),
            folders: vec![FolderGraph {
                name: "Sprites".to_string(),
                folders: vec![],
                files: Files::new(),
                path_to_parent: Some(ViewPathLocation::new("folders")),
            }],
        };

        assert_eq!(fgm.root, root);
        assert_eq!(
            *fgm.dirty_handler.resources_to_reserialize(),
            hashmap! {
                ViewPathLocation::new("folders/Sprites.yy") => DirtyState::New
            }
        );

        fgm.remove_empty_folder(&new_folder.path).unwrap();
        assert_eq!(fgm.root.folders, vec![]);
        assert_eq!(*fgm.dirty_handler.resources_to_reserialize(), hashmap![]);
        assert_eq!(*fgm.dirty_handler.resources_to_remove(), hashmap![]);

        // bit of nesting...
        let new_folder = fgm.new_folder_end(Vfs::root_folder(), "Sprites").unwrap();
        let subfolder = fgm.new_folder_end(&new_folder.path, "Npcs").unwrap();
        assert_eq!(
            *fgm.dirty_handler.resources_to_reserialize(),
            hashmap! {
                ViewPathLocation::new("folders/Sprites.yy") => DirtyState::New,
                ViewPathLocation::new("folders/Sprites/Npcs.yy") => DirtyState::New,
            }
        );
        assert_eq!(
            fgm.root.folders,
            vec![FolderGraph {
                name: "Sprites".to_string(),
                path_to_parent: Some(ViewPathLocation::new("folders")),
                folders: vec![FolderGraph {
                    name: "Npcs".to_string(),
                    path_to_parent: Some(ViewPathLocation::new("folders/Sprites.yy")),
                    folders: vec![],
                    files: Files::new(),
                }],
                files: Files::new(),
            }]
        );

        // removal test...
        assert_eq!(
            fgm.remove_empty_folder(&new_folder.path),
            Err(FolderGraphError::CannotRemoveFolder)
        );
        fgm.remove_empty_folder(&subfolder.path).unwrap();
        fgm.remove_empty_folder(&new_folder.path).unwrap();
        assert_eq!(*fgm.dirty_handler.resources_to_reserialize(), hashmap! {});
        assert_eq!(*fgm.dirty_handler.resources_to_remove(), hashmap! {});

        // add and then check removal...
        let new_folder = fgm.new_folder_end(Vfs::root_folder(), "Sprites").unwrap();
        let subfolder = fgm.new_folder_end(&new_folder.path, "Npcs").unwrap();

        let mut dummy0 = vec![];
        let mut dummy1 = vec![];
        fgm.serialize(&mut dummy0, &mut dummy1);

        assert_eq!(
            dummy0.into_iter().collect::<HashSet<_>>(),
            maplit::hashset![
                YypFolder {
                    folder_path: ViewPathLocation::new("folders/Sprites.yy"),
                    common_data: yy_typings::CommonData {
                        name: "Sprites".to_string(),
                        ..Default::default()
                    },
                },
                YypFolder {
                    folder_path: ViewPathLocation::new("folders/Sprites/Npcs.yy"),
                    common_data: yy_typings::CommonData {
                        name: "Npcs".to_string(),
                        ..Default::default()
                    },
                }
            ]
        );
        assert_eq!(dummy1, vec![]);

        fgm.remove_empty_folder(&subfolder.path).unwrap();
        fgm.remove_empty_folder(&new_folder.path).unwrap();
        assert_eq!(
            *fgm.dirty_handler.resources_to_remove(),
            maplit::hashmap! {
                ViewPathLocation::new("folders/Sprites.yy") => DirtyState::Edit,
                ViewPathLocation::new("folders/Sprites/Npcs.yy") => DirtyState::Edit,
            }
        );
        assert_eq!(*fgm.dirty_handler.resources_to_reserialize(), hashmap! {});
    }

    #[test]
    fn find_folder() {
        let mut vfs = Vfs::new("project");
        let new_folder = vfs.new_folder_end(Vfs::root_folder(), "Sprites").unwrap();

        let mut scr = Script::default();
        scr.set_name("Vec2".to_string());
        scr.set_parent_view_path(new_folder.clone());
        vfs.new_resource_end(&scr).unwrap();

        vfs.new_folder_end(&new_folder.path, "Another Subfolder")
            .unwrap();

        let found = vfs.get_folder(&new_folder.path).unwrap();

        assert_eq!(
            *found,
            FolderGraph {
                name: "Sprites".to_string(),
                path_to_parent: Some(ViewPathLocation::new("folders")),
                folders: vec![FolderGraph {
                    name: "Another Subfolder".to_string(),
                    path_to_parent: Some(ViewPathLocation::new("folders/Sprites.yy",),),
                    folders: vec![],
                    files: Files::new(),
                },],
                files: Files::with_vec(vec![FilesystemPath {
                    name: "Vec2".to_string(),
                    path: Path::new("scripts/Vec2/Vec2.yy").to_owned(),
                },],),
            }
        );
    }
}
