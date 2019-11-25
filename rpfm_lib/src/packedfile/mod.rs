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
Module with all the code to decode/encode/interact with the different type of `PackedFiles`.

This module contains all the code related with interacting with the different type of `PackedFiles`
you can find in a `PackFile`. Here, you can find some generic enums used by the different `PackedFiles`.

For encoding/decoding/proper manipulation of the data in each type of `PackedFile`, check their respective submodules
!*/

use rpfm_error::{Error, ErrorKind, Result};

use std::{fmt, fmt::Display};
use std::ops::Deref;

use crate::packedfile::image::Image;
use crate::packedfile::table::{db::DB, loc::Loc};
use crate::packedfile::text::Text;
use crate::packfile::packedfile::RawPackedFile;
use crate::schema::Schema;
use crate::SCHEMA;


pub mod image;
pub mod rigidmodel;
pub mod table;
pub mod text;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// This enum represents a ***decoded `PackedFile`***,
///
/// Keep in mind that, despite we having logic to recognize them, we can't decode many of them yet.
#[derive(PartialEq, Clone, Debug)]
pub enum DecodedPackedFile {
    Anim,
    AnimFragment,
    AnimPack,
    AnimTable,
    CEO,
    DB(DB),
    Image(Image),
    Loc(Loc),
    MatchedCombat,
    RigidModel,
    StarPos,
    Text(Text),
    Unknown,
}

/// This enum specifies the different types of `PackedFile` we can find in a `PackFile`.
///
/// Keep in mind that, despite we having logic to recognize them, we can't decode many of them yet.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PackedFileType {
    Anim,
    AnimFragment,
    AnimPack,
    AnimTable,
    CEO,
    DB,
    Image,
    Loc,
    MatchedCombat,
    RigidModel,
    StarPos,
    Text,
    Unknown,
}

//----------------------------------------------------------------//
// Implementations for `DecodedPackedFile`.
//----------------------------------------------------------------//

/// Implementation of `DecodedPackedFile`.
impl DecodedPackedFile {

    /// This function decodes a `RawPackedFile` into a `DecodedPackedFile`, returning it.
    pub fn decode(data: &RawPackedFile) -> Result<Self> {
        match PackedFileType::get_packed_file_type(data.get_path()) {
            PackedFileType::DB => {
                let schema = SCHEMA.lock().unwrap();
                match schema.deref() {
                    Some(schema) => {
                        let name = data.get_path().get(1).ok_or_else(|| Error::from(ErrorKind::DBTableIsNotADBTable))?;
                        let data = data.get_data()?;
                        let packed_file = DB::read(&data, name, &schema)?;
                        Ok(DecodedPackedFile::DB(packed_file))
                    }
                    None => Ok(DecodedPackedFile::Unknown),
                }
            }

            PackedFileType::Image => {
                let data = data.get_data()?;
                let packed_file = Image::read(&data)?;
                Ok(DecodedPackedFile::Image(packed_file))
            }

            PackedFileType::Loc => {
                let schema = SCHEMA.lock().unwrap();
                match schema.deref() {
                    Some(schema) => {
                        let data = data.get_data()?;
                        let packed_file = Loc::read(&data, &schema)?;
                        Ok(DecodedPackedFile::Loc(packed_file))
                    }
                    None => Ok(DecodedPackedFile::Unknown),
                }
            }

            PackedFileType::Text => {
                let data = data.get_data()?;
                let packed_file = Text::read(&data)?;
                Ok(DecodedPackedFile::Text(packed_file))
            }
            _=> Ok(DecodedPackedFile::Unknown)
        }
    }

    /// This function decodes a `RawPackedFile` into a `DecodedPackedFile`, returning it.
    pub fn decode_no_locks(data: &RawPackedFile, schema: &Schema) -> Result<Self> {
        match PackedFileType::get_packed_file_type(data.get_path()) {
            PackedFileType::DB => {
                let name = data.get_path().get(1).ok_or_else(|| Error::from(ErrorKind::DBTableIsNotADBTable))?;
                let data = data.get_data()?;
                let packed_file = DB::read(&data, name, &schema)?;
                Ok(DecodedPackedFile::DB(packed_file))
            }

            PackedFileType::Image => {
                let data = data.get_data()?;
                let packed_file = Text::read(&data)?;
                Ok(DecodedPackedFile::Text(packed_file))
            }

            PackedFileType::Loc => {
                let data = data.get_data()?;
                let packed_file = Loc::read(&data, &schema)?;
                Ok(DecodedPackedFile::Loc(packed_file))
            }

            PackedFileType::Text => {
                let data = data.get_data()?;
                let packed_file = Text::read(&data)?;
                Ok(DecodedPackedFile::Text(packed_file))
            }
            _=> Ok(DecodedPackedFile::Unknown)
        }
    }

    /// This function encodes a `DecodedPackedFile` into a `Vec<u8>`, returning it.
    pub fn encode(&self) -> Result<Vec<u8>> {
        match self {
            DecodedPackedFile::DB(data) => data.save(),
            DecodedPackedFile::Image(_) => unimplemented!(),
            DecodedPackedFile::Loc(data) => data.save(),
            DecodedPackedFile::Text(data) => data.save(),
            _=> unimplemented!(),
        }
    }
}

//----------------------------------------------------------------//
// Implementations for `PackedFileType`.
//----------------------------------------------------------------//

/// Display implementation of `PackedFileType`.
impl Display for PackedFileType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PackedFileType::Anim => write!(f, "Anim"),
            PackedFileType::AnimFragment => write!(f, "AnimFragment"),
            PackedFileType::AnimPack => write!(f, "AnimPack"),
            PackedFileType::AnimTable => write!(f, "AnimTable"),
            PackedFileType::CEO => write!(f, "CEO"),
            PackedFileType::DB => write!(f, "DB Table"),
            PackedFileType::Image => write!(f, "Image"),
            PackedFileType::Loc => write!(f, "Loc Table"),
            PackedFileType::MatchedCombat => write!(f, "Matched Combat"),
            PackedFileType::RigidModel => write!(f, "RigidModel"),
            PackedFileType::StarPos => write!(f, "StartPos"),
            PackedFileType::Text => write!(f, "Text"),
            PackedFileType::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Implementation of `PackedFileType`.
impl PackedFileType {

    /// This function returns the type of the `PackedFile` at the provided path.
    pub fn get_packed_file_type(path: &[String]) -> Self {
        if let Some(packedfile_name) = path.last() {

            // If it's in the "db" folder, it's a DB PackedFile (or you put something were it shouldn't be).
            if path[0] == "db" { PackedFileType::DB }

            // If it ends in ".loc", it's a localisation PackedFile.
            else if packedfile_name.ends_with(".loc") { PackedFileType::Loc }

            // If it ends in ".rigid_model_v2", it's a RigidModel PackedFile.
            else if packedfile_name.ends_with(".rigid_model_v2") { PackedFileType::RigidModel }

            // If it ends in any of these, it's a plain text PackedFile.
            else if packedfile_name.ends_with(".lua") ||
                    packedfile_name.ends_with(".xml") ||
                    packedfile_name.ends_with(".xml.shader") ||
                    packedfile_name.ends_with(".xml.material") ||
                    packedfile_name.ends_with(".variantmeshdefinition") ||
                    packedfile_name.ends_with(".environment") ||
                    packedfile_name.ends_with(".lighting") ||
                    packedfile_name.ends_with(".wsmodel") ||
                    packedfile_name.ends_with(".csv") ||
                    packedfile_name.ends_with(".tsv") ||
                    packedfile_name.ends_with(".inl") ||
                    packedfile_name.ends_with(".battle_speech_camera") ||
                    packedfile_name.ends_with(".bob") ||
                    packedfile_name.ends_with(".cindyscene") ||
                    packedfile_name.ends_with(".cindyscenemanager") ||
                    //packedfile_name.ends_with(".benchmark") || // This one needs special decoding/encoding.
                    packedfile_name.ends_with(".txt") { PackedFileType::Text }

            // If it ends in any of these, it's an image.
            else if packedfile_name.ends_with(".jpg") ||
                    packedfile_name.ends_with(".jpeg") ||
                    packedfile_name.ends_with(".tga") ||
                    packedfile_name.ends_with(".dds") ||
                    packedfile_name.ends_with(".png") { PackedFileType::Image }

            // Otherwise, we don't have a decoder for that PackedFile... yet.
            else { PackedFileType::Unknown }
        }

        // If we didn't got a name, it means something broke. Return none.
        else { PackedFileType::Unknown }
    }
}