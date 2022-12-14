use std::collections::HashMap;

use crate::interface::KeyValuePair;
use crate::storage::Transaction;
use crate::util::{
	build_byte_map, build_bytes, build_sized, build_usize_from_bytes, concat_bytes, Component,
};
use crate::{Error, SimpleTransaction};
use solomon_gremlin::structure::PropertyMap;
use solomon_gremlin::{GValue, Property, GID};

impl_repository!(PropertyRepository(Property));

fn build_property_value(value: &GValue) -> Vec<u8> {
	build_bytes(&[Component::GValueType(value), Component::GValue(value)]).unwrap()
}

impl<'a> PropertyRepository<'a> {
	/// The property()-step is used to add properties to the elements of the graph (sideEffect).
	/// Unlike addV() and addE(), property() is a full sideEffect step in that it does not return
	/// the property it created, but the element that streamed into it. Moreover, if property()
	/// follows an addV() or addE(), then it is "folded" into the previous step to enable vertex
	/// and edge creation with all its properties in one creation operation.
	pub async fn property(
		&self,
		tx: &mut Transaction,
		id: &GID,
		label: &GValue,
		value: &GValue,
	) -> Result<Property, Error> {
		let cf = self.cf();
		let val = build_property_value(value);
		let key = concat_bytes(vec![
			build_sized(Component::Gid(id)),
			build_sized(Component::GValue(label)),
		]);
		tx.set(cf, key.to_vec(), val).await.unwrap();
		let label = label.get::<String>().unwrap();
		Ok(Property::new(label, value.clone()))
	}

	/// Method to iterate the pairs of byte data
	fn iterate(&self, iterator: Vec<Result<KeyValuePair, Error>>) -> Result<PropertyMap, Error> {
		let mut map = HashMap::<String, Property>::new();
		iterator.iter().for_each(|p| {
			let (k, v) = p.as_ref().unwrap();
			// Handle deserializing and rebuild vertex stream
			let bytemap = &build_byte_map(vec!["vertex_id", "label"], k.to_vec());
			let label_bytes = bytemap.get("label").unwrap().to_vec();
			let label = String::from_utf8(label_bytes).unwrap();
			// Handle deserializing GValue
			let variant = build_usize_from_bytes(v[..1].to_vec());
			let value = GValue::from_bytes(variant, v[1..].to_vec());
			let property = Property::new(label.clone(), value);
			map.insert(label, property);
		});

		Ok(map)
	}

	/// Method to iterate the pairs of byte data with prefix as edge id
	pub async fn iterate_from_edge(
		&self,
		tx: &Transaction,
		edge_id: &GID,
	) -> Result<PropertyMap, Error> {
		let cf = self.cf();
		let prefix = build_sized(Component::Gid(edge_id));
		let iterator = tx.prefix_iterate(cf, prefix).await.unwrap();
		self.iterate(iterator)
	}

	pub async fn iterate_from_label(
		&self,
		tx: &Transaction,
		edge_id: &GID,
		label: &GValue,
	) -> Result<PropertyMap, Error> {
		let cf = self.cf();
		let prefix = concat_bytes(vec![
			build_sized(Component::Gid(edge_id)),
			build_sized(Component::GValue(label)),
		]);
		let iterator = tx.prefix_iterate(cf, prefix).await.unwrap();
		self.iterate(iterator)
	}
}
