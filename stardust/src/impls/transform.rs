use bevy::prelude::Transform;
use glam::{Vec3, Quat};
use crate::shared::serialisation::{ManualBitSerialisation, BitWriter, BitReader, BitstreamError};

impl ManualBitSerialisation for Transform {
    fn serialise(&self, writer: &mut impl BitWriter) {
        writer.allocate_bytes(40);
        self.translation.serialise(writer);
        self.rotation.serialise(writer);
        self.scale.serialise(writer);
    }

    fn deserialise(reader: &mut impl BitReader) -> Result<Self, BitstreamError> {
        let mut transform = Transform::default();
        transform.translation = Vec3::deserialise(reader)?;
        transform.rotation = Quat::deserialise(reader)?;
        transform.scale = Vec3::deserialise(reader)?;
        Ok(transform)
    }
}