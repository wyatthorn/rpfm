//---------------------------------------------------------------------------//
// Copyright (c) 2017-2019 Ismael Gutiérrez González. All rights reserved.
// 
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
// 
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with all the code to interact with infividual PackedFiles.

This module contains all the code related with the interaction with individual PackFiles, 
meaning the code that takes care of loading/writing their data from/to disk. 

You'll rarely have to touch anything here.
!*/

use std::io::prelude::*;
use std::io::{BufReader, Read, SeekFrom};
use std::fs::File;
use std::sync::{Arc, Mutex};

use crate::packfile::*;
use crate::packfile::compression::decompress_data;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// This struct represents a PackedFile in memory.
#[derive(Clone, Debug, PartialEq)]
pub struct PackedFile {

    /// The path of the `PackedFile` inside the `PackFile`.
    path: Vec<String>,

    /// Name of the original `PackFile` containing it. To know from where a `PackedFile` came when loading multiple PackFiles as one.
    packfile_name: String,

    /// The '*Last Modified Date*' of the `PackedFile`, encoded in `i64`. Only in PackFiles with the appropiate flag enabled..
    timestamp: i64,

    /// If the data should be compressed when saving it to disk. Only available from `PFHVersion::PFH5` onwards.
    should_be_compressed: bool,

    /// If the data should be encrypted when saving it to disk. If it should, it contains `Some(PFHVersion)`, being `PFHVersion` the one of the game this `PackedFile` is for.
    should_be_encrypted: Option<PFHVersion>,

    /// the data of the PackedFile. Use the getter/setter functions to interact with it.
    data: PackedFileData,
}

/// This enum represents the data of a `PackedFile`, in his current state.
#[derive(Clone, Debug)]
pub enum PackedFileData {

    /// The data is loaded to memory and the variant holds the data and info about the current state of the data (data, is_compressed, is_encrypted).
    OnMemory(Vec<u8>, bool, Option<PFHVersion>),

    /// The data is not loaded to memory and the variant holds the info needed to get the data loaded to memory on demand 
    /// (reader of the file, position of the start of the data, size of the data, is_compressed, is_encrypted).
    OnDisk(Arc<Mutex<BufReader<File>>>, u64, u32, bool, Option<PFHVersion>),
} 

//---------------------------------------------------------------------------//
//                       Enum & Structs Implementations
//---------------------------------------------------------------------------//

/// Implementation of `PackedFile`.
impl PackedFile {

    /// This function creates a new `PackedFile` from a `Vec<u8>` and some extra data.
    pub fn read_from_vec(
        path: Vec<String>,
        packfile_name: String,
        timestamp: i64,
        should_be_compressed: bool, 
        data: Vec<u8>
    ) -> Self {
        Self {
            path,
            packfile_name,
            timestamp,
            should_be_compressed,
            should_be_encrypted: None,
            data: PackedFileData::OnMemory(data, should_be_compressed, None),
        }
    }

    /// This function creates a new `PackedFile` from a another's `PackedFile`'s data, and some extra data. What an asshole.
    pub fn read_from_data(
        path: Vec<String>,
        packfile_name: String, 
        timestamp: i64,
        should_be_compressed: bool, 
        should_be_encrypted: Option<PFHVersion>, 
        data: PackedFileData
    ) -> Self {
        Self {
            path,
            packfile_name,
            timestamp,
            should_be_compressed,
            should_be_encrypted,
            data,
        }
    }

    /// This function creates a new `PackedFile` from a file in the filesystem.
    ///
    /// Keep in mind that you have to set the name of his `PackFile` if you add it to one.
    pub fn read_from_path(
        path_as_file: &Path,
        path_as_packed_file: Vec<String>,
    ) -> Result<Self> {
        let mut file = BufReader::new(File::open(&path_as_file)?);
        let mut data = vec![];
        file.read_to_end(&mut data)?;
        Ok(PackedFile::read_from_vec(path_as_packed_file, String::new(), get_last_modified_time_from_file(&file.get_ref()), false, data))
    }

    /// This function loads the data of a `PackedFile` to memory, if it isn't loaded already.
    pub fn load_data(&mut self) -> Result<()> {
        let data_on_memory = if let PackedFileData::OnDisk(ref file, position, size, is_compressed, is_encrypted) = self.data {
            let mut data = vec![0; size as usize];
            file.lock().unwrap().seek(SeekFrom::Start(position))?;
            file.lock().unwrap().read_exact(&mut data)?;
            PackedFileData::OnMemory(data, is_compressed, is_encrypted)
        } else { return Ok(()) };
        
        self.data = data_on_memory;
        Ok(())
    }

    /// This function returns the data of the `PackedFile` without loading it to memory.
    ///
    /// It's for those situations where you just need to check the data once, then forget about it.
    pub fn get_data(&self) -> Result<Vec<u8>> {
        match self.data {
            PackedFileData::OnMemory(ref data, is_compressed, is_encrypted) => {
                let mut data = data.to_vec();
                if is_encrypted.is_some() { data = decrypt_packed_file(&data); }
                if is_compressed { data = decompress_data(&data)?; }
                Ok(data)
            },
            PackedFileData::OnDisk(ref file, position, size, is_compressed, is_encrypted) => {
                let mut data = vec![0; size as usize];
                file.lock().unwrap().seek(SeekFrom::Start(position))?;
                file.lock().unwrap().read_exact(&mut data)?;
                if is_encrypted.is_some() { data = decrypt_packed_file(&data); }
                if is_compressed { Ok(decompress_data(&data)?) }
                else { Ok(data) }
            }
        }
    }

    /// This function returns the data of the provided `PackedFile` loading it to memory in the process if it isn't already loaded.
    ///
    /// It's for when you need to keep the data for multiple uses.
    pub fn get_data_and_keep_it(&mut self) -> Result<Vec<u8>> {
        let data = match self.data {
            PackedFileData::OnMemory(ref mut data, ref mut is_compressed, ref mut is_encrypted) => {
                if is_encrypted.is_some() { *data = decrypt_packed_file(&data); }
                if *is_compressed { *data = decompress_data(&data)?; }
                *is_compressed = false;
                *is_encrypted = None;
                return Ok(data.to_vec())
            },
            PackedFileData::OnDisk(ref file, position, size, is_compressed, is_encrypted) => {
                let mut data = vec![0; size as usize];
                file.lock().unwrap().seek(SeekFrom::Start(position))?;
                file.lock().unwrap().read_exact(&mut data)?;
                if is_encrypted.is_some() { data = decrypt_packed_file(&data); }
                if is_compressed { decompress_data(&data)? }
                else { data }
            }
        };

        self.data = PackedFileData::OnMemory(data.to_vec(), false, None);
        Ok(data)
    }

    /// This function returns the data of the provided `PackedFile` from memory. together with his state info.
    ///
    /// The data returned is `data, is_compressed, is_encrypted, should_be_compressed, should_be_encrypted`.
    pub fn get_data_and_info_from_memory(&mut self) -> Result<(&mut Vec<u8>, &mut bool, &mut Option<PFHVersion>, &mut bool, &mut Option<PFHVersion>)> {
        match self.data {
            PackedFileData::OnMemory(ref mut data, ref mut is_compressed, ref mut is_encrypted) => {
                Ok((data, is_compressed, is_encrypted, &mut self.should_be_compressed, &mut self.should_be_encrypted))
            },
            PackedFileData::OnDisk(_, _, _, _, _) => {
                Err(ErrorKind::PackedFileDataIsNotInMemory)?
            }
        }
    }

    /// This function replaces the data on the `PackedFile` with the provided one.
    pub fn set_data(&mut self, data: Vec<u8>) {
        self.data = PackedFileData::OnMemory(data, false, None);
    }

    /// This function returns the size of the data of the provided `PackedFile`.
    pub fn get_size(&self) -> u32 {
        match self.data {
            PackedFileData::OnMemory(ref data, _, _) => data.len() as u32,
            PackedFileData::OnDisk(_, _, size, _, _) => size,
        }
    }

    /// This function returns the current compression state of the provided `PackedFile`.
    pub fn get_compression_state(&self) -> bool {
        match self.data {
            PackedFileData::OnMemory(_, state, _) => state,
            PackedFileData::OnDisk(_, _, _, state, _) => state,
        }
    }

    /// This function returns if the `PackedFile` should be compressed or not.
    pub fn get_should_be_compressed(&self) -> bool{
        self.should_be_compressed
    }

    /// This function sets if the `PackedFile` should be compressed or not.
    pub fn set_should_be_compressed(&mut self, state: bool) {
        self.should_be_compressed = state;
    }

    /// This function returns the name of the PackFile this `PackedFile` belongs to.
    pub fn get_packfile_name(&self) -> &str {
        &self.packfile_name
    }

    /// This function sets the name of the PackFile this `PackedFile` belongs to.
    pub fn set_packfile_name(&mut self, name: &str) {
        self.packfile_name = name.to_owned();
    }

    /// This function returns if the `PackedFile` should be encrypted or not.
    ///
    /// If it should, it'll return the `PFHVersion` to encrypt to.
    pub fn get_should_be_encrypted(&self) -> &Option<PFHVersion> {
        &self.should_be_encrypted
    }

    /// This function sets if the `PackedFile` should be encrypted or not.
    pub fn set_should_be_encrypted(&mut self, state: Option<PFHVersion>) {
        self.should_be_encrypted = state;
    }

    /// This function returns the timestamp of the provided `PackedFile`.
    pub fn get_timestamp(&self) -> i64 {
        self.timestamp
    }

    /// This function sets the timestamp of the provided `PackedFile`.
    pub fn set_timestamp(&mut self, timestamp: i64) {
        self.timestamp = timestamp;
    }

    /// This function returns a reference to the path of the provided `PackedFile`.
    pub fn get_path(&self) -> &[String] {
        &self.path
    }

    /// This function sets the path of the provided `PackedFile`.
    ///
    /// This can fail if you pass it an empty path, so make sure you check the result.
    ///
    /// ***WARNING***: DON'T USE THIS IF YOUR PACKEDFILE IS INSIDE A PACKFILE. USE THE `move_packedfile` FUNCTION INSTEAD.
    pub fn set_path(&mut self, path: &[String]) -> Result<()> {
        if path.is_empty() { return Err(ErrorKind::EmptyInput)? }
        self.path = path.to_vec();
        Ok(())
    }
}

/// Implementation of `PartialEq` for `PackedFileData`.
impl PartialEq for PackedFileData {
    fn eq(&self, other: &PackedFileData) -> bool {
        match (self, other) {
            (
                &PackedFileData::OnMemory(ref data, is_compressed, is_encrypted), 
                &PackedFileData::OnMemory(ref data_2, is_compressed_2, is_encrypted_2)) => 
                    data == data_2 && 
                    is_compressed == is_compressed_2 &&
                    is_encrypted == is_encrypted_2,
            _ => false,
        }
    }
}