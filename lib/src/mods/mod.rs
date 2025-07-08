use std::{
    collections::{BTreeMap, HashMap},
    error::Error,
    fmt::{Debug, Display},
    fs::{File, read_dir},
    io::{Read, Result as IoResult, Seek},
    ops::Deref,
    path::{Component, Path, PathBuf},
};
use zip::{HasZipMetadata, ZipArchive, read::ZipFile, result::ZipError};

use crate::{
    maps::{MapManager, parser::MapElementParsingError, reader::MapReadError},
    mods::{
        maps::{BundledMap, MapMeta},
        meta::ModMeta,
    },
    playbacks::{Playback, PlaybackReadError},
    utils::{FromYaml, YamlReadError},
};


#[cfg(not(target_family = "wasm"))]
use dotnetdll::dll::DLLError;

#[cfg(not(target_family = "wasm"))]
use crate::mods::dll::BufferedDLL;

#[cfg(not(target_family = "wasm"))]
pub mod dll;
pub mod maps;
pub mod meta;

#[derive(Clone, Copy, Debug, Default)]
pub struct FileProviderError<T: Error>(pub T);

impl<T: Error> FileProviderError<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T: Error> Error for FileProviderError<T> {}

impl<T: Error> Display for FileProviderError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl<T: Error> From<T> for FileProviderError<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

pub trait FileProvider {
    type Err: Error;
    type Reader<'a>: Read + 'a
    where Self: 'a;
    fn get_file(
        &mut self,
        path: impl AsRef<Path>,
    ) -> Result<Self::Reader<'_>, FileProviderError<Self::Err>>;
    fn get_paths(&self) -> impl Iterator<Item = &Path>;
    fn get_file_bytes(
        &mut self,
        path: impl AsRef<Path>,
    ) -> Result<Vec<u8>, FileProviderError<Self::Err>>;
}

struct ZipBuf<R: Read + Seek> {
    zip: ZipArchive<R>,
}

impl<R: Read + Seek> ZipBuf<R> {
    fn new(zip: ZipArchive<R>) -> Self {
        ZipBuf { zip }
    }
}

impl<R: Read + Seek> FileProvider for ZipBuf<R> {
    type Err = ZipError;
    type Reader<'a>
        = ZipFile<'a>
    where R: 'a;

    fn get_file(
        &mut self,
        path: impl AsRef<Path>,
    ) -> Result<Self::Reader<'_>, FileProviderError<Self::Err>> {
        self.zip
            .by_name(path.as_ref().to_str().unwrap())
            .map_err(Into::into)
    }

    fn get_paths(&self) -> impl Iterator<Item = &Path> {
        self.zip.file_names().map(Path::new)
    }

    fn get_file_bytes(
        &mut self,
        path: impl AsRef<Path>,
    ) -> Result<Vec<u8>, FileProviderError<Self::Err>> {
        let mut file = self.zip.by_name(path.as_ref().to_str().unwrap())?;
        let mut buf = Vec::with_capacity(file.get_metadata().uncompressed_size as usize);

        file.read_to_end(&mut buf).map_err(ZipError::from)?;

        Ok(buf)
    }
}

struct DirBuf {
    paths: Vec<PathBuf>,
    root: PathBuf,
    opened_files: BTreeMap<PathBuf, File>,
}

impl DirBuf {
    fn new(path: impl AsRef<Path>) -> IoResult<Self> {
        let path = path.as_ref().to_owned();

        let mut paths = Vec::new();
        let mut to_check = vec![path.clone()];

        while let Some(path) = to_check.pop() {
            for entry in read_dir(path)? {
                let entry = entry?;
                paths.push(entry.path());

                if entry.metadata()?.is_dir() {
                    to_check.push(paths.last().unwrap().clone());
                }
            }
        }


        Ok(DirBuf {
            paths,
            root: path,
            opened_files: BTreeMap::new(),
        })
    }
}

impl DirBuf {
    fn get_path_impl(&mut self, path: impl AsRef<Path>) -> IoResult<&File> {
        let path = path.as_ref();
        if !self.opened_files.contains_key(path) {
            let file = File::open(self.root.join(path))?;
            self.opened_files.insert(path.to_owned(), file);
        }

        // Unwrap safe since we've ensured the file exists
        Ok(self.opened_files.get(path).unwrap())
    }
}


impl FileProvider for DirBuf {
    type Err = std::io::Error;
    type Reader<'a>
        = &'a File
    where Self: 'a;

    fn get_file(
        &mut self,
        path: impl AsRef<Path>,
    ) -> Result<Self::Reader<'_>, FileProviderError<Self::Err>> {
        self.get_path_impl(path).map_err(Into::into)
    }

    fn get_paths(&self) -> impl Iterator<Item = &Path> {
        self.paths.iter().map(PathBuf::deref)
    }

    fn get_file_bytes(
        &mut self,
        path: impl AsRef<Path>,
    ) -> Result<Vec<u8>, FileProviderError<Self::Err>> {
        let mut file = self.get_path_impl(path)?;
        let mut buf = Vec::with_capacity(file.metadata()?.len() as usize);


        file.read_to_end(&mut buf)?;
        Ok(buf)
    }
}

pub struct ModManager<T: FileProvider> {
    provider: T,
    mods: ModCollection,
}

impl<R: Read + Seek> ModManager<ZipBuf<R>> {
    pub fn from_reader(reader: R) -> Result<Self, ModReadError<ZipError>> {
        ZipArchive::new(reader)
            .map(ZipBuf::new)
            .map(Self::from_provider)
            .map_err(ModReadError::ProviderError)?
    }
}

impl ModManager<DirBuf> {
    pub fn from_path(root: impl AsRef<Path>) -> Result<Self, ModReadError<std::io::Error>> {
        DirBuf::new(root).map(Self::from_provider)?
    }
}

impl<T: FileProvider> ModManager<T> {
    fn from_provider(mut provider: T) -> Result<Self, ModReadError<T::Err>> {
        let mods = ModCollection::new(&mut provider)?;
        Ok(ModManager { provider, mods })
    }

    pub fn get_file<'a>(&'a mut self, path: &Path) -> Result<impl Read + 'a + use<'a, T>, T::Err> {
        self.provider.get_file(path).map_err(|e| e.0)
    }

    pub fn mods(&self) -> &[Mod] {
        self.mods.mods()
    }

    pub fn mods_mut(&mut self) -> &mut Vec<Mod> {
        self.mods.mods_mut()
    }

    pub fn collection(&self) -> &ModCollection {
        &self.mods
    }

    pub fn collection_mut(&mut self) -> &mut ModCollection {
        &mut self.mods
    }
}

pub struct ModCollection {
    pub mod_defs: Vec<Mod>,
    pub tutorials: HashMap<String, Playback>,
    pub maps: HashMap<String, BundledMap>,
}

impl ModCollection {
    fn new<T: FileProvider>(provider: &mut T) -> Result<Self, ModReadError<T::Err>> {
        let meta_file = provider.get_file("everest.yaml")?;

        let mod_metas = Vec::<ModMeta>::parse_from_reader(meta_file).unwrap();

        let root_idx = mod_metas.iter().position(|meta| {
            mod_metas.iter().all(|other_meta| {
                if meta.name == other_meta.name {
                    true
                } else {
                    meta.dependencies
                        .iter()
                        .any(|(dep_name, _)| dep_name == &other_meta.name)
                }
            })
        });

        let mods = mod_metas
            .into_iter()
            .enumerate()
            .map(|(i, m)| Mod::new(m, Some(i) == root_idx, provider))
            .collect();

        let mut tutorials = HashMap::new();
        let mut map_metas = HashMap::new();
        let mut map_bins = HashMap::new();

        let paths = provider
            .get_paths()
            .map(ToOwned::to_owned)
            .collect::<Box<[_]>>();
        'file_loop: for path in paths {
            if path.to_str().is_none() {
                eprintln!("[celeste_rs]: Non-UTF-8 path found: \"{path:?}\". Skipping...");
                continue;
            }

            // We don't care about directories which may be included in get_paths
            if path.extension().is_none() {
                continue;
            }

            let mut components = path.components().enumerate();

            // Get the top level directory of the path
            // This is something like 'Tutorials' or 'Maps'
            // This makes it easier to check how we should attempt to parse a file
            // If the file is in the mod root we just set it to '/'
            let top_level_dir;

            loop {
                match components.next() {
                    // We don't care about dir redirection stuff
                    Some((_, Component::Prefix(_)))
                    | Some((_, Component::RootDir))
                    | Some((_, Component::CurDir))
                    | Some((_, Component::ParentDir)) => {}

                    // If this is the either the first or second component
                    // we're in the mod root
                    Some((0, Component::Normal(component)))
                    | Some((1, Component::Normal(component))) => {
                        if Path::new(&component).extension().is_some() {
                            top_level_dir = "/";
                        } else {
                            top_level_dir = component.to_str().unwrap();
                        }
                        break;
                    }

                    // This is the top level directory
                    Some((_, Component::Normal(a))) => {
                        top_level_dir = a.to_str().unwrap();
                        break;
                    }

                    // We have an invalid path so just stop trying to use it
                    None => continue 'file_loop,
                }
            }

            if top_level_dir == "Tutorials" {
                let mut file = provider.get_file(&path)?;

                let tutorial = Playback::from_reader(&mut file)?;
                // Safe since we check we can convert to String
                tutorials.insert(path.to_str().unwrap().to_owned(), tutorial);
            } else if top_level_dir == "Maps" {
                let mut file = provider.get_file(&path)?;
                // Doesn't panic since we check starts_with
                let path = path.strip_prefix("Mods").unwrap();
                // We checked that extension returns Some so this will work
                let path_str = path.to_str().unwrap();

                // We ensure the path has an extension earlier
                let index_of_dot = path_str.find('.').unwrap();

                let sid = &path_str[.. index_of_dot];


                if path.ends_with(".bin") {
                    let mut mm = MapManager::new(&mut file)?;
                    mm.default_parsers();
                    let map = mm.parse_map()?;
                    // Safe since we check we can convert to String
                    map_bins.insert(sid.to_owned(), map);
                } else if path.ends_with(".altsideshelper.meta.yaml") {
                    // TODO: altsides helper stuff
                } else if path.ends_with(".meta.yaml") {
                    map_metas.insert(sid.to_owned(), MapMeta::parse_from_reader(&mut file)?);
                }
            }
        }

        let mut maps = HashMap::new();
        for (sid, map_meta) in map_metas {
            let map_bin = map_bins.remove(&sid).unwrap();
            maps.insert(sid.to_owned(), BundledMap {
                meta: map_meta,
                altsides_meta: None,
                map: map_bin,
            });
        }

        Ok(Self {
            mod_defs: mods,
            tutorials,
            maps,
        })
    }

    pub fn mods(&self) -> &[Mod] {
        &self.mod_defs
    }

    pub fn mods_mut(&mut self) -> &mut Vec<Mod> {
        &mut self.mod_defs
    }
}

pub struct Mod {
    pub meta: ModMeta,
    /// Whether this mod should be the one with all the non-dll assets connected to it
    pub root: bool,
    #[cfg(not(target_family = "wasm"))]
    pub dll: Option<BufferedDLL>,
}

impl Mod {
    #[cfg_attr(target_family = "wasm", allow(unused_variables))]
    fn new(meta: ModMeta, root: bool, provider: &mut impl FileProvider) -> Self {
        #[cfg(not(target_family = "wasm"))]
        let dll = if let Some(dll_path) = &meta.dll {
            let dll_bytes = provider.get_file_bytes(dll_path).unwrap();
            Some(BufferedDLL::new(dll_bytes).unwrap())
        } else {
            None
        };

        Mod {
            meta,
            root,

            #[cfg(not(target_family = "wasm"))]
            dll,
        }
    }
}

pub enum ModReadError<T: Error> {
    ProviderError(T),
    #[cfg(not(target_family = "wasm"))]
    DllError(DLLError),
    MapReadError(MapReadError),
    MapParseError(MapElementParsingError),
    IoError(std::io::Error),
    PlaybackError(PlaybackReadError),
    YamlReadError(YamlReadError),
}

impl<T: Error> Display for ModReadError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModReadError::ProviderError(e) => Display::fmt(e, f),
            #[cfg(not(target_family = "wasm"))]
            ModReadError::DllError(e) => Display::fmt(e, f),
            ModReadError::MapReadError(e) => Display::fmt(e, f),
            ModReadError::MapParseError(e) => Display::fmt(e, f),
            ModReadError::IoError(e) => Display::fmt(e, f),
            ModReadError::PlaybackError(e) => Display::fmt(e, f),
            ModReadError::YamlReadError(e) => Display::fmt(e, f),
        }
    }
}

impl<T: Error> From<FileProviderError<T>> for ModReadError<T> {
    fn from(value: FileProviderError<T>) -> Self {
        ModReadError::ProviderError(value.0)
    }
}

#[cfg(not(target_family = "wasm"))]
impl<T: Error> From<DLLError> for ModReadError<T> {
    fn from(value: DLLError) -> Self {
        ModReadError::<T>::DllError(value)
    }
}

impl<T: Error> From<MapReadError> for ModReadError<T> {
    fn from(value: MapReadError) -> Self {
        ModReadError::<T>::MapReadError(value)
    }
}

impl<T: Error> From<MapElementParsingError> for ModReadError<T> {
    fn from(value: MapElementParsingError) -> Self {
        ModReadError::<T>::MapParseError(value)
    }
}

impl<T: Error> From<std::io::Error> for ModReadError<T> {
    fn from(value: std::io::Error) -> Self {
        ModReadError::<T>::IoError(value)
    }
}

impl<T: Error> From<PlaybackReadError> for ModReadError<T> {
    fn from(value: PlaybackReadError) -> Self {
        ModReadError::<T>::PlaybackError(value)
    }
}

impl<T: Error> From<YamlReadError> for ModReadError<T> {
    fn from(value: YamlReadError) -> Self {
        ModReadError::<T>::YamlReadError(value)
    }
}
