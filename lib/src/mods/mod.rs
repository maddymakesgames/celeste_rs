use std::{
    collections::{BTreeMap, HashMap},
    fmt::Debug,
    fs::{read_dir, File},
    io::{Read, Result as IoResult, Seek},
    ops::Deref,
    path::{Path, PathBuf},
};
use zip::{
    read::ZipFile,
    result::{ZipError, ZipResult},
    HasZipMetadata,
    ZipArchive,
};

use crate::{
    mods::{dll::BufferedDLL, meta::ModMeta},
    playbacks::Playback,
    utils::YamlFile,
};

pub mod dll;
pub mod meta;

pub trait FileProvider {
    type Err: Debug;
    type Reader<'a>: Read + 'a
    where Self: 'a;
    fn get_path<'a>(&'a mut self, path: &str) -> Result<Self::Reader<'a>, Self::Err>;
    fn get_paths(&self) -> impl Iterator<Item = &str>;
    fn get_path_bytes(&mut self, path: &str) -> Result<Vec<u8>, Self::Err>;
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
    type Reader<'a> = ZipFile<'a> where R: 'a;

    fn get_path<'a>(&'a mut self, path: &str) -> Result<Self::Reader<'a>, Self::Err> {
        self.zip.by_name(path)
    }

    fn get_paths(&self) -> impl Iterator<Item = &str> {
        self.zip.file_names()
    }

    fn get_path_bytes(&mut self, path: &str) -> Result<Vec<u8>, Self::Err> {
        let mut file = self.zip.by_name(path)?;
        let mut buf = Vec::with_capacity(file.get_metadata().uncompressed_size as usize);

        file.read_to_end(&mut buf)?;

        Ok(buf)
    }
}

struct DirBuf {
    paths: Vec<PathBuf>,
    root: PathBuf,
    opened_files: BTreeMap<String, File>,
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
    fn get_path_impl<'a>(&'a mut self, path: &str) -> IoResult<&'a File> {
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
    type Reader<'a> = &'a File
    where Self: 'a;

    fn get_path<'a>(&'a mut self, path: &str) -> Result<Self::Reader<'a>, Self::Err> {
        self.get_path_impl(path)
    }

    fn get_paths(&self) -> impl Iterator<Item = &str> {
        self.paths
            .iter()
            .map(PathBuf::deref)
            // this is probably fine to use since I don't think
            // we care about non-utf8 shit anyway. We want &str
            .filter_map(Path::to_str)
    }

    fn get_path_bytes(&mut self, path: &str) -> Result<Vec<u8>, Self::Err> {
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
    pub fn from_reader(reader: R) -> ZipResult<Self> {
        ZipArchive::new(reader)
            .map(ZipBuf::new)
            .map(Self::from_provider)?
    }
}

impl ModManager<DirBuf> {
    pub fn from_path(root: impl AsRef<Path>) -> IoResult<Self> {
        DirBuf::new(root).map(Self::from_provider)?
    }
}

impl<T: FileProvider> ModManager<T> {
    fn from_provider(mut provider: T) -> Result<Self, T::Err> {
        let mods = ModCollection::new(&mut provider)?;
        Ok(ModManager { provider, mods })
    }

    pub fn get_file<'a>(&'a mut self, path: &str) -> Result<impl Read + 'a, T::Err> {
        self.provider.get_path(path)
    }

    pub fn mods(&self) -> &[Mod] {
        self.mods.mods()
    }

    pub fn mods_mut(&mut self) -> &mut Vec<Mod> {
        self.mods.mods_mut()
    }
}

pub struct ModCollection {
    pub mod_defs: Vec<Mod>,
    pub tutorials: HashMap<String, Playback>,
}

impl ModCollection {
    fn new<T: FileProvider>(provider: &mut T) -> Result<Self, T::Err> {
        let meta_file = provider.get_path("everest.yaml")?;

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

        let paths = provider
            .get_paths()
            .map(ToOwned::to_owned)
            .collect::<Box<[_]>>();
        for path in paths {
            if path.starts_with("Tutorial") {
                let mut file = provider.get_path(&path)?;

                let tutorial = Playback::from_reader(&mut file).unwrap();
                tutorials.insert(path.to_owned(), tutorial);
            }
        }

        Ok(Self {
            mod_defs: mods,
            tutorials,
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
    pub dll: Option<BufferedDLL>,
}

impl Mod {
    fn new(meta: ModMeta, root: bool, provider: &mut impl FileProvider) -> Self {
        let dll = if let Some(dll_path) = &meta.dll {
            let dll_bytes = provider.get_path_bytes(dll_path).unwrap();
            Some(BufferedDLL::new(dll_bytes).unwrap())
        } else {
            None
        };

        Mod { meta, root, dll }
    }
}
