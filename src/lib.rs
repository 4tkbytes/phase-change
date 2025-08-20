//! This crate aims to convert from one type of file to another. It is cheaper and easier (and local) to 
//! make the transformation locally than over the web. 

pub mod converters;

use std::{collections::{HashMap, HashSet, VecDeque}, path::PathBuf};

use crate::converters::{audio::AudioFileType, image::{png::PngToJpeg, ImageFileType}};

#[derive(Eq, Hash, PartialEq, Debug, Clone, Copy)]
pub enum FileType {
    Unknown,
    Image(ImageFileType),
    Audio(AudioFileType),
}

impl Default for FileType {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Default)]
pub struct FileConvertBuilder {
    from: (FileType, PathBuf),
    to: (FileType, Option<PathBuf>),
    registry: Option<ConverterRegistry>,
    custom_converters: Vec<Box<dyn Converter>>,
}

impl FileConvertBuilder {
    pub fn new() -> Self {
        Self {
            registry: Some(ConverterRegistry::new()),
            ..Default::default()
        }
    }

    pub fn from_file(&mut self, file_type: FileType, file_location: PathBuf) -> &mut Self {
        self.from = (file_type, file_location);
        self
    }

    pub fn to_file(&mut self, file_type: FileType, file_location: Option<PathBuf>) -> &mut Self {
        self.to = (file_type, file_location);
        self
    }

    pub fn with_converter<C>(&mut self, converter: C) -> &mut Self 
    where
        C: Converter + 'static,
    {
        self.custom_converters.push(Box::new(converter));
        self
    }

    pub fn with_converters(&mut self, converters: Vec<Box<dyn Converter>>) -> &mut Self {
        self.custom_converters.extend(converters);
        self
    }

    pub fn convert(mut self) -> anyhow::Result<()> {
        let mut registry = self.registry.take().ok_or_else(|| anyhow::anyhow!("No converter registry available"))?;
        
        for converter in self.custom_converters {
            registry.register(converter);
        }

        if self.from.0 == FileType::Unknown {
            return Err(anyhow::anyhow!("Source file type not specified"));
        }
        
        if self.to.0 == FileType::Unknown {
            return Err(anyhow::anyhow!("Target file type not specified"));
        }
        
        let output_path = match self.to.1 {
            Some(path) => path,
            None => {
                let mut output = self.from.1.clone();
                output.set_extension(get_extension_for_type(&self.to.0));
                output
            }
        };
        
        if registry.can_convert(self.from.0, self.to.0) {
            return registry.convert(&self.from.0, &self.to.0, &self.from.1, &output_path);
        }
        
        if let Some(path) = registry.find_conversion_path(self.from.0, self.to.0) {
            println!("Multi-step conversion path: {:?}", path);
            
            let mut current_input = self.from.1.clone();
            
            for window in path.windows(2) {
                if let [from_type, to_type] = window {
                    let temp_output = if to_type == &self.to.0 {
                        output_path.clone()
                    } else {
                        let mut temp = current_input.clone();
                        temp.set_extension(get_extension_for_type(to_type));
                        temp.set_file_name(format!("temp_{}", temp.file_name().unwrap().to_string_lossy()));
                        temp
                    };
                    
                    registry.convert(from_type, to_type, &current_input, &temp_output)?;
                    current_input = temp_output;
                }
            }
            
            return Ok(());
        }
        
        Err(anyhow::anyhow!("No conversion path available from {:?} to {:?}", self.from.0, self.to.0))
    }
}

fn get_extension_for_type(file_type: &FileType) -> &'static str {
    match file_type {
        FileType::Unknown => "unknown",
        FileType::Image(image_file_type) => {
            match image_file_type {
                ImageFileType::PNG => "png",
                ImageFileType::JPEG => "jpg",
            }
        },
        FileType::Audio(audio_file_type) => {
            match audio_file_type {
                AudioFileType::MP3 => "mp3",
                AudioFileType::WAV => "wav",
            }
        },
    }
}

pub trait Converter: Send + Sync {
    fn convert(&self, input_path: &PathBuf, output_path: &PathBuf) -> anyhow::Result<()>;
    fn from_type(&self) -> FileType;
    fn to_type(&self) -> FileType;
}

pub struct ConverterRegistry {
    converters: HashMap<(FileType, FileType), Box<dyn Converter>>,
}

impl ConverterRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            converters: HashMap::new(),
        };
        
        registry.register(Box::new(PngToJpeg));
        
        registry
    }

    pub fn register(&mut self, converter: Box<dyn Converter>) {
        let key = (converter.from_type(), converter.to_type());
        self.converters.insert(key, converter);
    }

    pub fn can_convert(&self, from: FileType, to: FileType) -> bool {
        self.converters.contains_key(&(from.clone(), to.clone()))
    }
    
    pub fn convert(&self, from: &FileType, to: &FileType, input: &PathBuf, output: &PathBuf) -> anyhow::Result<()> {
        let key = (from.clone(), to.clone());
        match self.converters.get(&key) {
            Some(converter) => converter.convert(input, output),
            None => Err(anyhow::anyhow!("No converter available from {:?} to {:?}", from, to)),
        }
    }

    pub fn find_conversion_path(&self, from: FileType, to: FileType) -> Option<Vec<FileType>> {
        if from == to {
            return Some(vec![from.clone()]);
        }
        
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        let mut parent: HashMap<FileType, FileType> = HashMap::new();
        
        queue.push_back(from.clone());
        visited.insert(from.clone());
        
        while let Some(current) = queue.pop_front() {
            if current == to {
                let mut path = vec![current.clone()];
                let mut node = current;
                
                while let Some(p) = parent.get(&node) {
                    path.push(p.clone());
                    node = p.clone();
                }
                
                path.reverse();
                return Some(path);
            }
            
            for ((from_type, to_type), _) in &self.converters {
                if *from_type == current && !visited.contains(to_type) {
                    visited.insert(to_type.clone());
                    parent.insert(to_type.clone(), current.clone());
                    queue.push_back(to_type.clone());
                }
            }
        }
        
        None
    }
}

