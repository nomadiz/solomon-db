use std::io::{Cursor, Error as IoError, Write};

use byteorder::{BigEndian, WriteBytesExt};
use chrono::{DateTime, NaiveDateTime, Timelike, Utc};

use lazy_static::lazy_static;
use rand::Rng;
use solomon_gremlin::{GValue, LabelType, GID};
use uuid::Uuid;

lazy_static! {
	/// The maximum possible datetime.
	pub static ref MAX_DATETIME: DateTime<Utc> =
		DateTime::from_utc(NaiveDateTime::from_timestamp_opt(i64::from(i32::MAX), 0).unwrap(), Utc)
			.with_nanosecond(1_999_999_999u32)
			.unwrap();
}

pub enum Component<'a> {
	Uuid(Uuid),
	/// GID: Gremlin Identifier
	Gid(&'a GID),
	/// GLabelType: Gremlin Label Type
	Label(&'a LabelType),
	FixedLengthString(&'a str),
	DateTime(DateTime<Utc>),
	Bytes(&'a [u8]),
	Usize(usize),
	/// GValue: Gremlin Value
	GValue(&'a GValue),
	/// GValueType: Gremlin Value Type
	GValueType(&'a GValue),
}

impl<'a> Component<'a> {
	pub fn len(&self) -> usize {
		match *self {
			Component::Uuid(_) => 16,
			Component::FixedLengthString(s) => s.len(),
			Component::DateTime(_) => 8,
			Component::Bytes(b) => b.len(),
			Component::GValue(v) | Component::GValueType(v) => v.bytes().len(),
			Component::Gid(v) => v.bytes_len(),
			Component::Label(v) => v.bytes_len(),
			Component::Usize(_) => 1,
		}
	}

	pub fn write(&self, cursor: &mut Cursor<Vec<u8>>) -> Result<(), IoError> {
		match *self {
			Component::Uuid(uuid) => cursor.write_all(uuid.as_bytes()),
			Component::FixedLengthString(s) => cursor.write_all(s.as_bytes()),
			Component::DateTime(datetime) => {
				let time_to_end = nanos_since_epoch(&MAX_DATETIME) - nanos_since_epoch(&datetime);
				cursor.write_u64::<BigEndian>(time_to_end)
			}
			Component::Bytes(bytes) => cursor.write_all(bytes),
			Component::GValueType(v) => cursor.write_all(&[v.to_variant()]),
			Component::GValue(value) => cursor.write_all(value.bytes().as_slice()),
			Component::Gid(value) => cursor.write_all(value.bytes().as_slice()),
			Component::Label(value) => cursor.write_all(value.bytes().as_slice()),
			Component::Usize(value) => cursor.write_all(&[value.try_into().unwrap()]),
		}
	}

	pub fn _read_uuid(bytes: &[u8]) -> Result<Uuid, IoError> {
		let mut fix: [u8; 16] = Default::default();
		fix.copy_from_slice(&bytes[0..16]);
		Ok(Uuid::from_bytes(fix))
	}
}

/// Gets the number of nanoseconds since unix epoch for a given datetime.
fn nanos_since_epoch(datetime: &DateTime<Utc>) -> u64 {
	let timestamp = datetime.timestamp() as u64;
	let nanoseconds = u64::from(datetime.timestamp_subsec_nanos());
	timestamp * 1_000_000_000 + nanoseconds
}

pub fn generate_random_i32() -> i32 {
	let mut rng = rand::thread_rng();
	rng.gen::<i32>()
}
