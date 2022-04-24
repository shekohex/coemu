use bytes::{Buf, Bytes};
use encoding::all::ASCII;
use encoding::{DecoderTrap, Encoding};
use std::io::{BufReader, Read};
use std::path::PathBuf;
use std::{env, fs};

fn main() -> Result<(), String> {
    dotenv::dotenv().map_err(|e| e.to_string())?;
    let data_path = env::var("DATA_LOCATION").map_err(|e| e.to_string())?;
    let dat_path = PathBuf::from(data_path)
        .join("GameMaps")
        .join("GameMap.dat");
    let f = fs::File::open(dat_path).map_err(|e| e.to_string())?;
    let mut buffer = Vec::new();
    let mut reader = BufReader::with_capacity(1024, f);
    reader.read_to_end(&mut buffer).map_err(|e| e.to_string())?;
    let mut buffer = Bytes::from(buffer);
    let amount = buffer.get_u32_le();
    for _ in 0..amount {
        let map_id = buffer.get_u32_le();
        let path_len = buffer.get_u32_le() as usize;
        let path_buf = buffer.split_to(path_len);
        let mut path = ASCII
            .decode(&path_buf, DecoderTrap::Ignore)
            .expect("Never fails");
        path.replace_range(0..8, "");
        let path = path.replace(".DMap", ".cmap");
        println!(
            r#"INSERT INTO maps VALUES ({map_id}, '{path}', 0, 0, 0) ON CONFLICT (map_id) DO UPDATE SET path = '{path}', revive_point_x = 0, revive_point_y = 0, flags = 0;"#,
            map_id = map_id,
            path = path
        );
        buffer.advance(4); // puzzle
    }

    Ok(())
}
