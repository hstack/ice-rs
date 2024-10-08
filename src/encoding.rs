use crate::protocol::*;
use crate::errors::*;
use std::convert::TryInto;
use std::collections::HashMap;
use std::hash::Hash;
use num_enum::TryFromPrimitive;
use std::convert::TryFrom;

/// The `IceSize` is a special wrapper around i32
/// and is encoded in one byte if smaller than 255.
#[derive(Debug)]
pub struct IceSize {
    pub size: i32
}

/// The `ToBytes` trait needs to be implemented by all types
/// that need to be encoded.
pub trait ToBytes {
    fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn std::error::Error + Sync + Send>>;
}

/// The `FromBytes` trait needs to be implemented by all types
/// that need to be decoded.
pub trait FromBytes {
    fn from_bytes(bytes: &[u8], read_bytes: &mut i32) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> where Self: Sized;
}

pub trait OptionalType {
    fn optional_type() -> u8;
}

impl OptionalType for bool {
    fn optional_type() -> u8 {
        0
    }
}

impl OptionalType for u8 {
    fn optional_type() -> u8 {
        0
    }
}

impl OptionalType for i8 {
    fn optional_type() -> u8 {
        0
    }
}

impl OptionalType for u16 {
    fn optional_type() -> u8 {
        1
    }
}

impl OptionalType for i16 {
    fn optional_type() -> u8 {
        1
    }
}

impl OptionalType for u32 {
    fn optional_type() -> u8 {
        2
    }
}

impl OptionalType for i32 {
    fn optional_type() -> u8 {
        2
    }
}

impl OptionalType for f32 {
    fn optional_type() -> u8 {
        2
    }
}

impl OptionalType for u64 {
    fn optional_type() -> u8 {
        3
    }
}

impl OptionalType for i64 {
    fn optional_type() -> u8 {
        3
    }
}

impl OptionalType for f64 {
    fn optional_type() -> u8 {
        3
    }
}


impl OptionalType for String {
    fn optional_type() -> u8 {
        5
    }
}

pub struct OptionalWrapper<T> {
    flag: OptionalFlag,
    value: Option<T>
}

impl<T: OptionalType> OptionalWrapper<T> {
    pub fn new(tag: u8, value: Option<T>) -> OptionalWrapper<T> {
        OptionalWrapper {
            flag: OptionalFlag::new(tag, T::optional_type()),
            value
        }
    }
}

impl<T> Into<Option<T>> for OptionalWrapper<T> {
    fn into(self) -> Option<T> {
        self.value
    }
}

#[derive(Clone, Debug, Eq, PartialEq, TryFromPrimitive)]
#[repr(u8)]
pub enum SliceFlagsTypeEncoding {
    NoTypeId,
    StringTypeId,
    IndexTypeId,
    CompactTypeId,
}

#[derive(Debug)]
pub struct SliceFlags {
    pub type_id: SliceFlagsTypeEncoding,
    pub optional_members: bool,
    pub indirection_table: bool,
    pub slice_size: bool,
    pub last_slice: bool,
}

#[derive(Debug)]
pub struct OptionalFlag {
    pub tag: u8,
    pub r#type: u8
}

impl OptionalFlag {
    pub fn new(tag: u8, r#type: u8) -> OptionalFlag {
        OptionalFlag {
            tag: tag,
            r#type: r#type
        }
    }
}

// BASIC ENCODING FUNCTIONS
impl ToBytes for SliceFlags {
    fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn std::error::Error + Sync + Send>> {
        let mut byte = self.type_id.clone() as u8 & 0b11;
        if self.optional_members {
            byte = byte | 0b100;
        }
        if self.indirection_table {
            byte = byte | 0b1000;
        }
        if self.slice_size {
            byte = byte | 0b10000;
        }
        if self.last_slice {
            byte = byte | 0b100000;
        }

        Ok(vec![byte])
    }
}

impl FromBytes for SliceFlags {
    fn from_bytes(bytes: &[u8], read_bytes: &mut i32) -> Result<Self, Box<dyn std::error::Error + Sync + Send>>
    where Self: Sized {
        if bytes.len() < 1 {
            return Err(Box::new(ProtocolError::new("Not enough bytes to read SliceFlags")));
        } else {
            let byte = bytes[0];
            *read_bytes = *read_bytes + 1;
            Ok(SliceFlags {
                type_id: SliceFlagsTypeEncoding::try_from(byte & 0b11)?,
                optional_members: byte & 0b100 > 0,
                indirection_table: byte & 0b1000 > 0,
                slice_size: byte & 0b10000 > 0,
                last_slice: byte & 0b100000 > 0,
            })
        }
    }
}

impl ToBytes for OptionalFlag {
    fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn std::error::Error + Sync + Send>> {
        let tag_bits = self.r#type & 7;
        let type_bits = self.tag << 3;
        Ok(vec![tag_bits | type_bits])
    }
}

impl FromBytes for OptionalFlag {
    fn from_bytes(bytes: &[u8], read_bytes: &mut i32) -> Result<Self, Box<dyn std::error::Error + Sync + Send>>
    where Self: Sized {
        if bytes.len() < 1 {
            return Err(Box::new(ProtocolError::new("Not enough bytes to read OptionalFlag")));
        } else {
            let byte = bytes[0];
            let tag = byte >> 3;
            let r#type = byte & 7;
            *read_bytes = *read_bytes + 1;
            Ok(OptionalFlag::new(tag, r#type))
        }
    }
}

impl ToBytes for IceSize {
    fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn std::error::Error + Sync + Send>> {
        if self.size < 255 {
            Ok(vec![self.size as u8])
        } else {
            let mut bytes = vec![255];
            bytes.extend(self.size.to_bytes()?);
            Ok(bytes)
        }
    }
}

impl FromBytes for IceSize {
    fn from_bytes(bytes: &[u8], read_bytes: &mut i32) -> Result<Self, Box<dyn std::error::Error + Sync + Send>>
    where Self: Sized {
        if bytes.len() < 1 {
            return Err(Box::new(ProtocolError::new("Not enough bytes to read IceSize")));
        }
        else if bytes[0] == 255 {
            if bytes.len() < 5 {
                return Err(Box::new(ProtocolError::new("Not enough bytes to read IceSize")));
            } else {
                *read_bytes = 1;
                Ok(IceSize {
                    size: i32::from_bytes(&bytes[1..5], read_bytes)?
                })
            }
        } else {
            Ok(IceSize {
                size: u8::from_bytes(bytes, read_bytes)? as i32
            })
        }
    }
}

impl ToBytes for str {
    fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn std::error::Error + Sync + Send>> {
        let mut bytes = IceSize{size: self.len() as i32}.to_bytes()?;
        bytes.extend(self.as_bytes());
        Ok(bytes)
    }
}

impl ToBytes for String {
    fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn std::error::Error + Sync + Send>> {
        let mut bytes = IceSize{size: self.len() as i32}.to_bytes()?;
        bytes.extend(self.as_bytes());
        Ok(bytes)
    }
}

impl FromBytes for String {
    fn from_bytes(bytes: &[u8], read_bytes: &mut i32) -> Result<Self, Box<dyn std::error::Error + Sync + Send>>
    where Self: Sized {
        let mut read = 0;
        let size = IceSize::from_bytes(bytes, &mut read)?.size;
        let s = String::from_utf8(bytes[read as usize..read as usize + size as usize].to_vec())?;
        *read_bytes = *read_bytes + read + size;
        Ok(s)
    }
}


impl<T: ToBytes, U: ToBytes> ToBytes for HashMap<T, U> {
    fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn std::error::Error + Sync + Send>> {
        let mut bytes = IceSize{size: self.len() as i32}.to_bytes()?;
        for (key, value) in self {
            bytes.extend(key.to_bytes()?);
            bytes.extend(value.to_bytes()?);
        }
        Ok(bytes)
    }
}

impl<T: FromBytes + Eq + Hash, U: FromBytes> FromBytes for HashMap<T, U> {
    fn from_bytes(bytes: &[u8], read_bytes: &mut i32) -> Result<Self, Box<dyn std::error::Error + Sync + Send>>
    where Self: Sized {
        let mut read = 0;
        let size = IceSize::from_bytes(bytes, &mut read)?.size;
        let mut dict: HashMap<T, U> = HashMap::new();

        for _i in 0..size {
            let key = T::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?;
            let value = U::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?;
            dict.insert(key, value);
        }
        *read_bytes = *read_bytes + read;
        Ok(dict)
    }
}

impl<T: ToBytes> ToBytes for Vec<T>
{
    fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn std::error::Error + Sync + Send>> {
        let mut bytes = IceSize{size: self.len() as i32}.to_bytes()?;
        for item in self {
            bytes.extend(item.to_bytes()?);
        }
        Ok(bytes)
    }
}

impl<T: FromBytes> FromBytes for Vec<T>
{
    fn from_bytes(bytes: &[u8], read_bytes: &mut i32) -> Result<Self, Box<dyn std::error::Error + Sync + Send>>
    where Self: Sized {
        let mut read = 0;
        let size = IceSize::from_bytes(bytes, &mut read)?.size;
        let mut seq: Vec<T> = vec![];

        for _i in 0..size {
            seq.push(T::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?);
        }
        *read_bytes = *read_bytes + read;
        Ok(seq)
    }
}

impl ToBytes for u8 {
    fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn std::error::Error + Sync + Send>> {
        Ok(vec![*self])
    }
}

impl FromBytes for u8 {
    fn from_bytes(bytes: &[u8], read_bytes: &mut i32) -> Result<Self, Box<dyn std::error::Error + Sync + Send>>
    where Self: Sized {
        *read_bytes = *read_bytes + 1;
        Ok(bytes[0])
    }
}

impl ToBytes for i16 {
    fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn std::error::Error + Sync + Send>> {
        Ok(self.to_le_bytes().to_vec())
    }
}

impl FromBytes for i16 {
    fn from_bytes(bytes: &[u8], read_bytes: &mut i32) -> Result<Self, Box<dyn std::error::Error + Sync + Send>>
    where Self: Sized {
        let size = std::mem::size_of::<i16>();
        if bytes.len() < size {
            return Err(Box::new(ProtocolError::new("Not enough bytes to read i16")));
        }
        match bytes[0..size].try_into() {
            Ok(barray) => {
                *read_bytes = *read_bytes + size as i32;
                Ok(i16::from_le_bytes(barray))
            },
            _ => Err(Box::new(ProtocolError::new("Error reading i16")))
        }
    }
}

impl ToBytes for i32 {
    fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn std::error::Error + Sync + Send>> {
        Ok(self.to_le_bytes().to_vec())
    }
}

impl FromBytes for i32 {
    fn from_bytes(bytes: &[u8], read_bytes: &mut i32) -> Result<Self, Box<dyn std::error::Error + Sync + Send>>
    where Self: Sized {
        let size = std::mem::size_of::<i32>();
        if bytes.len() < size {
            return Err(Box::new(ProtocolError::new("Not enough bytes to read i32")));
        }
        match bytes[0..size].try_into() {
            Ok(barray) => {
                *read_bytes = *read_bytes + size as i32;
                Ok(i32::from_le_bytes(barray))
            },
            _ => Err(Box::new(ProtocolError::new("Error reading i32")))
        }
    }
}

impl ToBytes for i64 {
    fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn std::error::Error + Sync + Send>> {
        Ok(self.to_le_bytes().to_vec())
    }
}

impl FromBytes for i64 {
    fn from_bytes(bytes: &[u8], read_bytes: &mut i32) -> Result<Self, Box<dyn std::error::Error + Sync + Send>>
    where Self: Sized {
        let size = std::mem::size_of::<i64>();
        if bytes.len() < size {
            return Err(Box::new(ProtocolError::new("Not enough bytes to read i64")));
        }
        match bytes[0..size].try_into() {
            Ok(barray) => {
                *read_bytes = *read_bytes + size as i32;
                Ok(i64::from_le_bytes(barray))
            },
            _ => Err(Box::new(ProtocolError::new("Error reading i64")))
        }
    }
}

impl ToBytes for f32 {
    fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn std::error::Error + Sync + Send>> {
        Ok(self.to_le_bytes().to_vec())
    }
}

impl FromBytes for f32 {
    fn from_bytes(bytes: &[u8], read_bytes: &mut i32) -> Result<Self, Box<dyn std::error::Error + Sync + Send>>
    where Self: Sized {
        let size = std::mem::size_of::<f32>();
        if bytes.len() < size {
            return Err(Box::new(ProtocolError::new("Not enough bytes to read f32")));
        }
        match bytes[0..size].try_into() {
            Ok(barray) => {
                *read_bytes = *read_bytes + size as i32;
                Ok(f32::from_le_bytes(barray))
            },
            _ => Err(Box::new(ProtocolError::new("Error reading f32")))
        }
    }
}

impl ToBytes for f64 {
    fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn std::error::Error + Sync + Send>> {
        Ok(self.to_le_bytes().to_vec())
    }
}

impl FromBytes for f64 {
    fn from_bytes(bytes: &[u8], read_bytes: &mut i32) -> Result<Self, Box<dyn std::error::Error + Sync + Send>>
    where Self: Sized {
        let size = std::mem::size_of::<f64>();
        if bytes.len() < size {
            return Err(Box::new(ProtocolError::new("Not enough bytes to read f64")));
        }
        match bytes[0..size].try_into() {
            Ok(barray) => {
                *read_bytes = *read_bytes + size as i32;
                Ok(f64::from_le_bytes(barray))
            },
            _ => Err(Box::new(ProtocolError::new("Error reading f64")))
        }
    }
}

impl ToBytes for bool {
    fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn std::error::Error + Sync + Send>> {
        Ok(vec![if *self { 1 } else { 0 }])
    }
}

impl ToBytes for () {
    fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn std::error::Error + Sync + Send>> {
        Ok(vec![])
    }
}

impl FromBytes for bool {
    fn from_bytes(bytes: &[u8], read_bytes: &mut i32) -> Result<Self, Box<dyn std::error::Error + Sync + Send>>
    where Self: Sized {
        if bytes.len() < 1 {
            return Err(Box::new(ProtocolError::new("Not enough bytes to read bool")));
        }
        *read_bytes = *read_bytes + 1;
        Ok(bytes[0] != 0)
    }
}


impl ToBytes for Encapsulation {
    fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn std::error::Error + Sync + Send>>
    {
        let mut buffer: Vec<u8> = Vec::new();
        buffer.extend(&self.size.to_le_bytes());
        buffer.push(self.major);
        buffer.push(self.minor);
        if self.data.len() > 0 {
            buffer.extend(&self.data);
        }
        Ok(buffer)
    }
}

impl FromBytes for Encapsulation {
    fn from_bytes(bytes: &[u8], read_bytes: &mut i32) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        let mut read: i32 = 0;
        if bytes.len() < 6 {
            return Err(Box::new(ProtocolError::new("Not enough bytes to read Encapsulation")));
        }

        let size = i32::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?;
        let major = u8::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?;
        let minor = u8::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?;
        *read_bytes = *read_bytes + read + (bytes.len() as i32 - read);

        Ok(Encapsulation {
            size: size,
            major: major,
            minor: minor,
            data: bytes[read as usize..bytes.len()].to_vec()
        })
    }
}

impl FromBytes for ReplyData {
    fn from_bytes(bytes: &[u8], read_bytes: &mut i32) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        let mut read: i32 = 0;
        if bytes.len() < 11 {
            return Err(Box::new(ProtocolError::new("Not enough bytes to read ReplyData")));
        }

        let request_id = i32::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?;
        let status = u8::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?;
        match status {
            0 | 1 => {
                let encapsulation = Encapsulation::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?;
                *read_bytes = *read_bytes + read;
                Ok(ReplyData {
                    request_id: request_id,
                    status: status,
                    body: encapsulation
                })
            }
            // 1 => {
            //     Err(Error::EncapsulatedUserException(Encapsulation::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?))
            // }
            7 => {
                Err(Box::new(
                    RemoteException {
                        cause: String::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?
                    }
                ))
            }
            _ => Err(Box::new(ProtocolError::new(&format!("Unsupported ReplyData status: {}", status))))
        }
    }
}

impl ToBytes for Header {
    fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn std::error::Error + Sync + Send>>
    {
        let mut buffer: Vec<u8> = Vec::new();
        buffer.extend(self.magic.as_bytes());
        buffer.extend(self.protocol_major.to_bytes()?);
        buffer.extend(self.protocol_minor.to_bytes()?);
        buffer.extend(self.encoding_major.to_bytes()?);
        buffer.extend(self.encoding_minor.to_bytes()?);
        buffer.extend(self.message_type.to_bytes()?);
        buffer.extend(self.compression_status.to_bytes()?);
        buffer.extend(self.message_size.to_bytes()?);

        Ok(buffer)
    }
}

impl FromBytes for Header {
    fn from_bytes(bytes: &[u8], read_bytes: &mut i32) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        if bytes.len() < 14 {
            return Err(Box::new(ProtocolError::new("Not enough bytes to read Header")));
        }

        let magic = String::from_utf8(bytes[0..4].to_vec())?;
        if magic != "IceP" {
            return Err(Box::new(ProtocolError::new(&format!("Wrong magic! Expected IceP but found {}", magic))));
        }
        let mut read: i32 = 4;
        let protocol_major = u8::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?;
        let protocol_minor = u8::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?;
        let encoding_major = u8::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?;
        let encoding_minor = u8::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?;
        let message_type = u8::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?;
        let comression_status = u8::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?;
        let message_size = i32::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?;
        *read_bytes = *read_bytes + read;

        Ok(Header {
            magic: magic,
            protocol_major: protocol_major,
            protocol_minor: protocol_minor,
            encoding_major: encoding_major,
            encoding_minor: encoding_minor,
            message_type: message_type,
            compression_status: comression_status,
            message_size: message_size
        })
    }
}

impl<T: ToBytes> ToBytes for Option<T> {
    fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn std::error::Error + Sync + Send>> {
        let mut bytes = Vec::new();
        match self {
            Some(value) => {
                // TODO: use optional flag here
                bytes.extend((11 as u8).to_bytes()?);
                bytes.extend(value.to_bytes()?);
            }
            None => {}
        }
        Ok(bytes)
    }
}

impl<T: FromBytes> FromBytes for Option<T> {
    fn from_bytes(bytes: &[u8], read_bytes: &mut i32) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        if bytes.len() > 0 {
            let mut read: i32 = 0;
            let _flag = u8::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?;
            let result = Some(T::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?);
            *read_bytes = *read_bytes + read;
            Ok(result)
        } else {
            Ok(None)
        }
    }
}

impl<T: ToBytes> ToBytes for OptionalWrapper<T> {
    fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn std::error::Error + Sync + Send>> {
        let mut bytes = Vec::new();
        match &self.value {
            Some(value) => {
                bytes.extend(self.flag.to_bytes()?);
                bytes.extend(value.to_bytes()?);
            }
            None => {}
        }
        Ok(bytes)
    }
}

impl<T: FromBytes> FromBytes for OptionalWrapper<T> {
    fn from_bytes(bytes: &[u8], read_bytes: &mut i32) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        if bytes.len() > 0 {
            let mut read: i32 = 0;
            let result = OptionalWrapper {
                flag: OptionalFlag::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?,
                value: Some(T::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?)
            };
            *read_bytes = *read_bytes + read;
            Ok(result)
        } else {
            Ok(OptionalWrapper {
                flag: OptionalFlag::new(0, 0),
                value: None
            })
        }
    }
}

impl FromBytes for LocatorResult {
    fn from_bytes(bytes: &[u8], read_bytes: &mut i32) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        let mut read = 0;

        let proxy_data = ProxyData::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?;
        let size = IceSize::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?;

        if size.size == 0 {
            return Err(Box::new(ProtocolError::new("Error reading LocatorResult")));
        }

        match u8::from_bytes(&bytes[read as usize..bytes.len()], &mut read)? {
            1 => {
                let _unsued = i16::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?;
                let encapsulation = Encapsulation::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?;
                let tcp = EndpointData::from_bytes(&encapsulation.data[0..encapsulation.data.len()], &mut read)?;
                *read_bytes = *read_bytes + read;
                Ok(LocatorResult {
                    proxy_data: proxy_data,
                    size: size,
                    endpoint: EndPointType::TCP(tcp)
                })
            },
            2 => {
                let _unsued = i16::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?;
                let encapsulation = Encapsulation::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?;
                let ssl = EndpointData::from_bytes(&encapsulation.data[0..encapsulation.data.len()], &mut read)?;
                *read_bytes = *read_bytes + read;
                Ok(LocatorResult {
                    proxy_data: proxy_data,
                    size: size,
                    endpoint: EndPointType::SSL(ssl)
                })
            }
            0 => {
                let object = String::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?;
                *read_bytes = *read_bytes + read;
                Ok(LocatorResult {
                    proxy_data: proxy_data,
                    size: size,
                    endpoint: EndPointType::WellKnownObject(object)
                })
            }
            _ => {
                Err(Box::new(ProtocolError::new("Unsupported endpoint type")))
            }
        }
    }
}



#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_size_encoding() {
        let mut read_bytes = 0;
        let encoded = IceSize{size: 10}.to_bytes().expect("Could not encode size");
        let decoded = IceSize::from_bytes(&encoded, &mut read_bytes).expect("Could not decode size").size;
        assert_eq!(10, decoded);
        assert_eq!(1, read_bytes);

        read_bytes = 0;
        let encoded = IceSize{size: 500}.to_bytes().expect("Could not encode size");
        let decoded = IceSize::from_bytes(&encoded, &mut read_bytes).expect("Could not decode size").size;
        assert_eq!(500, decoded);
        assert_eq!(5, read_bytes);
    }

    #[test]
    fn test_string_encoding() {
        let mut read_bytes = 0;
        let encoded = "Hello".to_bytes().expect("Cannot necode test string");
        let decoded = String::from_bytes(&encoded, &mut read_bytes).expect("Cannot decode test string");
        assert_eq!("Hello", decoded);
        assert_eq!(6, read_bytes);
    }

    #[test]
    fn test_dict_encoding() {
        let mut read_bytes = 0;
        let mut dict = HashMap::new();
        dict.insert(String::from("Hello"), String::from("World"));

        let encoded = dict.to_bytes().expect("Cannot encode test dict");
        let decoded: HashMap<String, String> = HashMap::from_bytes(&encoded, &mut read_bytes).expect("Cannot decode test dict");
        assert!(decoded.contains_key("Hello"));
        assert_eq!("World", decoded.get("Hello").unwrap_or(&String::from("")));
    }

    #[test]
    fn test_string_seq_encoding() {
        let mut read_bytes = 0;
        let seq = vec![String::from("Hello"), String::from("World")];
        let encoded = seq.to_bytes().expect("Cannot encode test dict");
        let decoded: Vec<String> = Vec::from_bytes(&encoded, &mut read_bytes).expect("Cannot decode test dict");
        assert_eq!(2, decoded.len());
        assert_eq!(seq, decoded);
    }

    #[test]
    fn test_short_encoding() {
        let mut read_bytes = 0;
        let value: i16 = 3;
        let encoded = value.to_bytes().expect("Cannot encode test short");
        let decoded = i16::from_bytes(&encoded, &mut read_bytes).expect("Cannot decode test short");
        assert_eq!(value, decoded);
    }

    #[test]
    fn test_int_encoding() {
        let mut read_bytes = 0;
        let value: i32 = 3;
        let encoded = value.to_bytes().expect("Cannot encode test int");
        let decoded = i32::from_bytes(&encoded, &mut read_bytes).expect("Cannot decode test int");
        assert_eq!(value, decoded);
    }

    #[test]
    fn test_long_encoding() {
        let mut read_bytes = 0;
        let value: i64 = 3;
        let encoded = value.to_bytes().expect("Cannot encode test long");
        let decoded = i64::from_bytes(&encoded, &mut read_bytes).expect("Cannot decode test long");
        assert_eq!(value, decoded);
    }

    #[test]
    fn test_float_encoding() {
        let mut read_bytes = 0;
        let value: f32 = 3.14;
        let encoded = value.to_bytes().expect("Cannot encode test float");
        let decoded = f32::from_bytes(&encoded, &mut read_bytes).expect("Cannot decode test float");
        assert_eq!(value, decoded);
    }

    #[test]
    fn test_double_encoding() {
        let mut read_bytes = 0;
        let value: f64 = 3.14;
        let encoded = value.to_bytes().expect("Cannot encode test double");
        let decoded = f64::from_bytes(&encoded, &mut read_bytes).expect("Cannot decode double long");
        assert_eq!(value, decoded);
    }

    #[test]
    fn test_bool_encoding() {
        let mut read_bytes = 0;
        let value = true;
        let encoded = value.to_bytes().expect("Cannot encode test bool");
        let decoded = bool::from_bytes(&encoded, &mut read_bytes).expect("Cannot decode test bool");
        assert_eq!(value, decoded);
    }

    #[test]
    fn test_identity_ecoding() {
        let mut read_bytes = 0;
        let id = Identity {
            name: String::from("Hello"),
            category: String::from(""),
        };
        let bytes = id.to_bytes().expect("Cannot encode test identity");
        let decoded = Identity::from_bytes(&bytes, &mut read_bytes).expect("Cannot decode test identity");
        assert_eq!(7, read_bytes);
        assert_eq!(id.name, decoded.name);
        assert_eq!(id.category, decoded.category);
    }

    #[test]
    fn test_header_ecoding() {
        let mut read_bytes = 0;
        let header = Header::new(0, 14);
        let bytes = header.to_bytes().expect("Cannot encode test header");
        let decoded = Header::from_bytes(&bytes, &mut read_bytes).expect("Cannot decode test header");
        assert_eq!(14, read_bytes);
        assert_eq!(header.magic, decoded.magic);
        assert_eq!(header.message_size, decoded.message_size);
        assert_eq!(header.message_type, decoded.message_type);
        assert_eq!(header.magic, decoded.magic);
    }

    #[test]
    fn test_request_ecoding() {
        let mut read_bytes = 0;
        let request = RequestData {
            request_id: 1,
            id: Identity {
                name: String::from("Test"),
                category: String::from(""),
            },
            facet: vec![],
            operation: String::from("Op"),
            mode: 0,
            context: HashMap::new(),
            params: Encapsulation::empty()
        };
        let bytes = request.to_bytes().expect("Cannot encode test request");
        let decoded = RequestData::from_bytes(&bytes, &mut read_bytes).expect("Cannot decode test request");
        assert_eq!(22, read_bytes);
        assert_eq!(request.request_id, decoded.request_id);
        assert_eq!(request.id.name, decoded.id.name);
        assert_eq!(request.facet, decoded.facet);
        assert_eq!(request.operation, decoded.operation);
        assert_eq!(request.mode, decoded.mode);
        assert_eq!(request.context, decoded.context);
    }

    #[test]
    fn test_reply_encoding() {
        let mut read_bytes = 0;
        let reply = ReplyData {
            request_id: 1,
            status: 0,
            body: Encapsulation::empty()
        };
        let bytes = reply.to_bytes().expect("Cannot encode test reply");
        let decoded = ReplyData::from_bytes(&bytes, &mut read_bytes).expect("Cannot decode test reply");
        assert_eq!(11, read_bytes);
        assert_eq!(reply.request_id, decoded.request_id);
        assert_eq!(reply.status, decoded.status);
    }
}