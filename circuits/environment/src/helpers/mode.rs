// Copyright (C) 2019-2022 Aleo Systems Inc.
// This file is part of the snarkVM library.

// The snarkVM library is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The snarkVM library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with the snarkVM library. If not, see <https://www.gnu.org/licenses/>.

use crate::ParserResult;
use snarkvm_utilities::{error, FromBytes, ToBytes};

use core::fmt;
use nom::{branch::alt, bytes::complete::tag, combinator::map};
use num_traits::{FromPrimitive, ToPrimitive};
use std::io::{Read, Result as IoResult, Write};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, FromPrimitive, ToPrimitive)]
pub enum Mode {
    Constant,
    Public,
    Private,
}

impl Mode {
    /// Returns `true` if the mode is a constant.
    pub fn is_constant(&self) -> bool {
        matches!(self, Self::Constant)
    }

    /// Returns `true` if the mode is public.
    pub fn is_public(&self) -> bool {
        matches!(self, Self::Public)
    }

    /// Returns `true` if the mode is private.
    pub fn is_private(&self) -> bool {
        matches!(self, Self::Private)
    }

    /// Parses the string for the mode.
    pub fn parse(string: &str) -> ParserResult<Self> {
        alt((
            map(tag("constant"), |_| Self::Constant),
            map(tag("public"), |_| Self::Public),
            map(tag("private"), |_| Self::Private),
        ))(string)
    }
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Constant => write!(f, "constant"),
            Self::Public => write!(f, "public"),
            Self::Private => write!(f, "private"),
        }
    }
}

impl ToBytes for Mode {
    fn write_le<W: Write>(&self, mut writer: W) -> IoResult<()> {
        u8::write_le(&ToPrimitive::to_u8(self).ok_or_else(|| error("Invalid mode"))?, &mut writer)
    }
}

impl FromBytes for Mode {
    fn read_le<R: Read>(mut reader: R) -> IoResult<Self> {
        let mode = u8::read_le(&mut reader)?;
        FromPrimitive::from_u8(mode).ok_or_else(|| error("Invalid mode"))
    }
}
