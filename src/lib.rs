use bevy::{asset::Handle, sprite::ColorMaterial};

pub mod polygon;
pub mod eu4;
pub mod province;
pub mod border_segment;

pub const LAND_MATERIAL_HANDLE: Handle<ColorMaterial> = Handle::weak_from_u128(0xf00_4befa6c0e7f11d40d8931715303ac);
pub const BORDER_MATERIAL_HANDLE: Handle<ColorMaterial> = Handle::weak_from_u128(0xf02_4befa6c0e7f11d40d8931715303ac);
pub const SELECTED_BORDER_MATERIAL_HANDLE: Handle<ColorMaterial> = Handle::weak_from_u128(0xf03_4befa6c0e7f11d40d8931715303ac);
pub const SEA_MATERIAL_HANDLE: Handle<ColorMaterial> = Handle::weak_from_u128(0xf04_4befa6c0e7f11d40d8931715303ac);
pub const SELECTED_PROV_MATERIAL_HANDLE: Handle<ColorMaterial> = Handle::weak_from_u128(0xf05_4befa6c0e7f11d40d8931715303ac);