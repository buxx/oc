pub fn extract<T>(
    iter: impl Iterator<Item = T>,
    tileset: &tiled::Tileset,
) -> Result<Vec<(T, u32)>, String>
where
    T: std::fmt::Display + Eq + std::hash::Hash,
{
    iter.map(|item| {
        match tileset.tiles().find(|(_, tile)| {
            tile.properties.get("ID") == Some(&tiled::PropertyValue::StringValue(item.to_string()))
        }) {
            Some((id, _)) => Ok((item, id)),
            None => Err(format!("Missing tile for {item}")),
        }
    })
    .collect::<Result<Vec<(T, u32)>, String>>()
}
