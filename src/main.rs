type Xs = [u32; 4];

fn xorshift(xs: &mut Xs) -> u32 {
    let mut t = xs[3];

    xs[1] = xs[0];
    xs[2] = xs[1];
    xs[3] = xs[2];

    t ^= t << 11;
    t ^= t >> 8;
    xs[0] = t ^ xs[0] ^ (xs[0] >> 19);
    xs[0]
}

fn xs_u32(xs: &mut Xs, min: u32, max: u32) -> u32 {
    (xorshift(xs) % (max - min)) + min
}

fn xs_choice_str<'a>(xs: &mut Xs, slice: &[&'a str]) -> &'a str {
    let len = slice.len();
    assert!(len < u32::max_value() as _);
    slice[xs_u32(xs,0, len as u32) as usize]
}

fn xs_choice(xs: &mut Xs, slice: &[u32]) -> u32 {
    let len = slice.len();
    assert!(len < u32::max_value() as _);
    slice[xs_u32(xs, 0, len as u32) as usize]
}

use std::io::{SeekFrom, prelude::*};
use std::{fs, fs::OpenOptions};
use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use std::path::Path;
use std::ffi::CString;

#[derive(Serialize, Deserialize)]
struct Room {
    entrances: Vec<u32>,
    items: Vec<u32>,
    warp_ptrs: Vec<u32>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input_path = "./Paper Mario (USA).z64";
    let output_path = "./Paper Mario (USA) Shuffled.z64";

    enum Mode {
        Default,
        QuickStart
    }

    let mut args = std::env::args();
    //exe name
    args.next();

    let mode = match args.next() {
        Some(s) => {
            let s: &str = &s;
            match s {
                "--quick-start"|"-q" => Mode::QuickStart,
                _ => Mode::Default
            }
        }
        _ => Mode::Default
    };

    fs::copy(input_path, output_path)?;

    let mut output = OpenOptions::new()
        .read(true)
        .write(true)
        .open(output_path)?;

    let room_data: HashMap<&str, Room> = serde_json::from_str(include_str!("roomdata.json"))?;

    let mut names_to_skip = HashSet::with_capacity(12);
    names_to_skip.insert("end_00");
    names_to_skip.insert("end_01");
    names_to_skip.insert("gv_01");
    names_to_skip.insert("mgm_03");
    names_to_skip.insert("tst_11");
    names_to_skip.insert("tst_12");
    names_to_skip.insert("tst_13");
    names_to_skip.insert("tst_20");

    names_to_skip.insert("hos_04");
    names_to_skip.insert("hos_05");
    names_to_skip.insert("hos_10");
    names_to_skip.insert("mac_05");

    let room_names = room_data.keys()
        .filter(|name| {
            !names_to_skip.contains(*name)
        })
        .map(|x| *x)
        .collect::<Vec<&str>>();

    // TODO automatically endian-convert if needed.
    // make sure this is a paper mario rom with proper endianness
    {
        output.seek(SeekFrom::Start(0x20))?;
        let mut buf = [0u8; 11];
        output.read_exact(&mut buf)?;
        assert_eq!(&buf, b"PAPER MARIO", "Endian and/or rom type mismatch!");
    }

    #[allow(unused_assignments)]
    {
        let mut offset = 0x808A8;
        macro_rules! seek_write_advance {
            ($data: expr) => {
                output.seek(SeekFrom::Start(offset))?;
                output.write(&($data as u32).to_be_bytes())?;

                offset += 4;
            };
        }
        // code patch: start with Goombario out
        seek_write_advance!(0xA0820012);
        seek_write_advance!(0xA082000A); // enable action command
        seek_write_advance!(0x2402FFFF);
        offset = 0x808E4;
        seek_write_advance!(0xA0800000);

        // have every party member
        seek_write_advance!(0xA0A20014);

        // enable menus
        offset = 0x168074;
        seek_write_advance!(0x2406FF81);

        if let Mode::QuickStart = mode {
            // don't start the game from Mario's house
            offset = 0x168080;
            seek_write_advance!(0x24020000);
        }
    }

    let room_base_ptr = 0x80240000;
    let room_count = 421;
    let xs: &mut Xs = &mut [42,42,42,42];
    let mut already_randomized = HashMap::with_capacity(room_count as usize / 2);
    for i in 0..room_count {
        output.seek(SeekFrom::Start(0x6B450 + i*0x20))?;

        macro_rules! read_u32 {
            () => {{
                let mut buf = [0u8; 4];
                output.read_exact(&mut buf)?;
                u32::from_be_bytes(buf)
            }};
        }
        let name_ptr = read_u32!() - 0x80024C00;

        output.seek(SeekFrom::Current(4))?;
        let room_ptr = read_u32!();

        output.seek(SeekFrom::Start(name_ptr as _))?;
        let name_buf = {
            const SIZE: usize = 8;
            let mut name_buf = [0u8; SIZE];
            output.read_exact(&mut name_buf)?;

            let mut null_location = 0;
            for i in 0..SIZE {
                if name_buf[i] == 0 {
                    null_location = i;
                    break;
                }
            }
            std::ffi::CString::new(&name_buf[0..null_location])?
        };
        let name = name_buf.to_str()?;

        for warp_ptr in room_data[name].warp_ptrs.iter() {
            let rand_room;
            if already_randomized.contains_key(&warp_ptr) {
                rand_room = already_randomized[&warp_ptr]
            } else {
                rand_room = xs_choice_str(xs, &room_names);
            }
            already_randomized.insert(warp_ptr, rand_room);

            let rand_entrance = xs_choice(xs, &room_data[rand_room].entrances);

            output.seek(SeekFrom::Start((room_ptr + warp_ptr - room_base_ptr + 0xC) as _))?;
            let warp_room_ptr = read_u32!();
            output.write(&rand_entrance.to_be_bytes())?;
            output.seek(SeekFrom::Start((room_ptr + warp_room_ptr - room_base_ptr) as _))?;
            output.write(CString::new(rand_room)?.as_bytes_with_nul())?;
        }

        for item_ptr in room_data[name].items.iter() {
            output.seek(SeekFrom::Start((room_ptr + item_ptr - room_base_ptr) as _))?;
            let rand_item = xs_u32(xs, 1, 0x16C);
            let read_u32 = read_u32!();
            if 0 < read_u32 && read_u32 < 0x200 {
                output.seek(SeekFrom::Current(-4))?;
            }
            output.write(&rand_item.to_be_bytes())?;
        }
    }

    output.sync_data()?;
    drop(output);

    let rn64crc_path = Path::new("rn64crc\\rn64crc.exe");
    assert!(rn64crc_path.exists(), "rn64crc not found! File was written but the crc was not set.");

    std::process::Command::new(rn64crc_path)
        .args(&[output_path, "-u"])
        .output()
        .expect("failed to execute rn64crc");

    Ok(())
}
