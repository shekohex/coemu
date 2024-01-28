use bytes::{Buf, Bytes};
use std::collections::HashMap;
use std::io::{BufReader, Read};
use std::path::PathBuf;
use std::{env, fs};

#[allow(unused)]
#[derive(Debug, serde::Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
struct Map {
    uid: u32,
    name: String,
    #[serde(skip)]
    path: String,
    id: u32,
    flags: u32,
    weather: u8,
    portal_x: u16,
    portal_y: u16,
    reborn_map: u32,
    color: u32,
}

impl Default for Map {
    fn default() -> Self {
        Self {
            uid: 0,
            name: String::new(),
            path: String::new(),
            id: 0,
            flags: 0,
            weather: 0,
            portal_x: 0,
            portal_y: 0,
            reborn_map: 0,
            color: 4294967295,
        }
    }
}

#[derive(Debug, serde::Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
struct Portal {
    id: u32,
    from_map_id: u32,
    from_x: u16,
    from_y: u16,
    to_map_id: u32,
    to_x: u16,
    to_y: u16,
}

fn main() -> anyhow::Result<()> {
    dotenvy::dotenv()?;
    let data_path = env::var("DATA_LOCATION")?;
    let dat_path = PathBuf::from(&data_path).join("GameMaps").join("GameMap.dat");
    let maps_csv = PathBuf::from(&data_path).join("Maps").join("Maps.csv");
    let portals_csv = PathBuf::from(&data_path).join("Maps").join("Portals.csv");
    let csv_reader = csv::ReaderBuilder::new().has_headers(true).from_path(maps_csv)?;
    let mut maps = csv_reader
        .into_deserialize::<Map>()
        .filter_map(Result::ok)
        .map(|m| (m.uid, m))
        .collect::<HashMap<_, _>>();
    let csv_reader = csv::ReaderBuilder::new().has_headers(true).from_path(portals_csv)?;
    let portals = csv_reader
        .into_deserialize::<Portal>()
        .filter_map(Result::ok)
        .collect::<Vec<_>>();
    let f = fs::File::open(dat_path)?;
    let mut buffer = Vec::new();
    let mut reader = BufReader::with_capacity(1024, f);
    reader.read_to_end(&mut buffer)?;
    let mut buffer = Bytes::from(buffer);
    let amount = buffer.get_u32_le();
    let mut map_with_path = HashMap::with_capacity(amount as usize);
    for _ in 0..amount {
        let map_id = buffer.get_u32_le();
        let path_len = buffer.get_u32_le() as usize;
        let path_buf = buffer.split_to(path_len);
        let mut path = String::from_utf8(path_buf.into())?;
        path.replace_range(0..8, "");
        let path = path.replace(".DMap", ".cmap");
        map_with_path.insert(map_id, path);
        buffer.advance(4); // puzzle
    }
    assert!(map_with_path.len() >= amount as usize);
    // For maps without a path, we need to repair these
    // by using the path of the map with the same id but having a different uid.
    for (_, map) in maps.iter_mut() {
        if map.path.is_empty() {
            if let Some(path) = map_with_path.get(&map.id) {
                map.path = path.clone();
            }
        }
    }
    for Map {
        uid,
        id,
        path,
        flags,
        weather,
        portal_x,
        portal_y,
        reborn_map,
        color,
        ..
    } in maps.values()
    {
        if path.is_empty() {
            println!(
                r#"-- INSERT INTO maps VALUES ({uid}, {id}, '{path}', {portal_x}, {portal_y}, {flags}, {weather}, {reborn_map}, {color});"#,
            );
            continue;
        }
        println!(
            r#"INSERT INTO maps VALUES ({uid}, {id}, '{path}', {portal_x}, {portal_y}, {flags}, {weather}, {reborn_map}, {color});"#,
        );
    }
    println!();
    for Portal {
        id,
        from_map_id,
        from_x,
        from_y,
        to_map_id,
        to_x,
        to_y,
    } in portals
    {
        match (maps.get(&from_map_id), maps.get(&to_map_id)) {
            (Some(from), Some(to)) if !from.path.is_empty() && !to.path.is_empty() => {
                println!(
                    r#"INSERT INTO portals VALUES ({id}, {from_map_id}, {from_x}, {from_y}, {to_map_id}, {to_x}, {to_y});"#,
                );
            },
            _ => {
                println!(
                    r#"-- INSERT INTO portals VALUES ({id}, {from_map_id}, {from_x}, {from_y}, {to_map_id}, {to_x}, {to_y});"#,
                );
            },
        }
    }

    Ok(())
}
