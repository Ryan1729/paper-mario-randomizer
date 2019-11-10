type Xs = [core::num::Wrapping<u32>; 4];

fn xorshift(xs: &mut Xs) -> u32 {
    let mut t = xs[3];

    xs[3] = xs[2];
    xs[2] = xs[1];
    xs[1] = xs[0];

    t ^= t << 11;
    t ^= t >> 8;
    xs[0] = t ^ xs[0] ^ (xs[0] >> 19);

    xs[0].0
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

fn xs_shuffle(xs: &mut Xs, v: &mut Vec<u32>) {
    let len = v.len();
    if len < 2 {
        return;
    }
    for i in 0..(len - 2) {
        // Oh gee this won't shuffle the whole vec if there are ever 2^32 elements. Meh.
        let j = xs_u32(xs,i as u32, len as u32) as usize;
        v.swap(i, j);
    }
}

#[test]
fn xs_test() {
    let xs: &mut Xs = &mut [Wrapping(42),Wrapping(42),Wrapping(42),Wrapping(42)];
    for _ in 0..1000 {
        xs_u32(xs, 1, 0x16C);
    }

    for _ in 0..10 {
        xs_u32(xs, 1, 0x16C);
        dbg!(&xs);
    }
    panic!()
}

use std::io::{SeekFrom, prelude::*};
use std::{fs, fs::OpenOptions};
use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use std::path::Path;
use std::ffi::CString;
use std::num::Wrapping;

#[derive(Serialize, Deserialize)]
struct Room {
    entrances: Vec<u32>,
    items: Vec<u32>,
    warp_ptrs: Vec<u32>,
}

macro_rules! d {
    () => {
        Default::default();
    };
    (for $type: ty : $value: expr) => {
        impl Default for $type {
            fn default() -> Self {
                $value
            }
        }
    };
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input_path = "./Paper Mario (USA).z64";
    let output_path = "./Paper Mario (USA) Shuffled.z64";

    #[derive(Copy, Clone, PartialEq, Eq)]
    enum RoomMode {
        None,
        StartWithHammer,
        TotalRandom
    }
    d!(for RoomMode : RoomMode::StartWithHammer);

    enum StartMode {
        Standard,
        Quick
    }
    d!(for StartMode : StartMode::Standard);

    #[derive(PartialEq, Eq)]
    enum ItemMode {
        None,
        TotalRandom,
        ShuffleBadges,
        ShuffleUsedBadges,
    }
    d!(for ItemMode : ItemMode::None);

    let mut args = std::env::args();
    //exe name
    args.next();

    let mut start = d!();
    let mut item_mode = d!();
    let mut room_mode = d!();

    const SHUFFLE_ITEMS: &'static str = "--shuffle-items";
    const SHUFFLE_BADGES: &'static str = "--shuffle-badges";
    const SHUFFLE_USED_BADGES: &'static str = "--shuffle-used-badges";

    macro_rules! set_item_mode {
        ($mode: expr) => {{
            if item_mode != d!() {
                eprintln!(
                    "Only one of {:?} may be used.",
                     [SHUFFLE_ITEMS, SHUFFLE_BADGES, SHUFFLE_USED_BADGES]
                 );
                std::process::exit(2)
            }
            item_mode = $mode;
        }};
    }

    const TOTAL_ROOM_SHUFFLE: &'static str = "--total-room-shuffle";
    const NO_ROOM_SHUFFLE: &'static str = "--no-room-shuffle";

    macro_rules! set_room_mode {
        ($mode: expr) => {{
            if room_mode != d!() {
                eprintln!(
                    "Only one of {:?} may be used.",
                     [TOTAL_ROOM_SHUFFLE, NO_ROOM_SHUFFLE]
                 );
                std::process::exit(3)
            }
            room_mode = $mode;
        }};
    }

    const HELP: &'static str = "--help";
    const QUICK_START: &'static str = "--quick-start";

    while let Some(s) = args.next() {
        let s: &str = &s;
        match s {
            HELP => {
                let accepted_args = [
                    HELP,
                    QUICK_START,
                    SHUFFLE_ITEMS,
                    SHUFFLE_BADGES,
                    SHUFFLE_USED_BADGES,
                    TOTAL_ROOM_SHUFFLE,
                    NO_ROOM_SHUFFLE
                ];
                println!("reads {}, writes to {}", input_path, output_path);
                println!("accepted args: ");
                for arg in accepted_args.iter() {
                    println!("    {}", arg);
                }
                std::process::exit(0)
            },
            QUICK_START => start = StartMode::Quick,
            SHUFFLE_ITEMS => set_item_mode!(ItemMode::TotalRandom),
            SHUFFLE_BADGES => set_item_mode!(ItemMode::ShuffleBadges),
            SHUFFLE_USED_BADGES => set_item_mode!(ItemMode::ShuffleUsedBadges),
            NO_ROOM_SHUFFLE => set_room_mode!(RoomMode::None),
            TOTAL_ROOM_SHUFFLE => set_room_mode!(RoomMode::TotalRandom),
            _ => {
                eprintln!("unknown arg {:?}", s);
                std::process::exit(1)
            }
        }
    }

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

    let mut room_names = room_data.keys()
        .filter(|name| {
            !names_to_skip.contains(*name)
        })
        .map(|x| *x)
        .collect::<Vec<&str>>();
    room_names.sort();

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

        if let StartMode::Quick = start {
            // don't start the game from Mario's house
            offset = 0x168080;
            seek_write_advance!(0x24020000);
        }
    }

    macro_rules! read_u32 {
        () => {{
            let mut buf = [0u8; 4];
            output.read_exact(&mut buf)?;
            u32::from_be_bytes(buf)
        }};
    }

    let room_base_ptr: u32 = 0x80240000;

    // Set the pointed to exit in the pointed to room at the target room and entrance
    macro_rules! write_out_room_entrance {
        (
            $room_ptr: expr,
            $warp_ptr: expr,
            $target_room_name: expr,
            $target_room_entrance: expr
        ) => {
            let room_ptr: u32 = $room_ptr;
            let warp_ptr: u32 = $warp_ptr;
            let target_room_name: &str = $target_room_name;
            let target_room_entrance: u32 = $target_room_entrance;
            output.seek(SeekFrom::Start((room_ptr + warp_ptr - room_base_ptr + 0xC) as _))?;
            let warp_room_ptr = read_u32!();
            output.write(&target_room_entrance.to_be_bytes())?;
            output.seek(SeekFrom::Start((room_ptr + warp_room_ptr - room_base_ptr) as _))?;
            output.write(CString::new(target_room_name)?.as_bytes_with_nul())?;
        };
    }

    let xs: &mut Xs = &mut [Wrapping(43),Wrapping(42),Wrapping(42),Wrapping(42)];

    enum ItemState {
        None,
        TotalRandom,
        BadgeDeck(Vec<u32>)
    }

    let mut item_state = match item_mode {
        ItemMode::None => ItemState::None,
        ItemMode::TotalRandom => ItemState::TotalRandom,
        ItemMode::ShuffleBadges => {
            let mut deck = get_badges_set().into_iter().collect();
            xs_shuffle(xs, &mut deck);
            ItemState::BadgeDeck(deck)
        },
        ItemMode::ShuffleUsedBadges => {
            let mut deck = get_used_badges();
            xs_shuffle(xs,&mut deck);
            ItemState::BadgeDeck(deck)
        },
    };

    let room_count = 421;
    for i in 0..room_count {
        output.seek(SeekFrom::Start(0x6B450 + i * 0x20))?;

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

        match room_mode {
            RoomMode::None => {},
            RoomMode::TotalRandom | RoomMode::StartWithHammer => {
                for &warp_ptr in room_data[name].warp_ptrs.iter() {
                    let rand_room = xs_choice_str(xs, &room_names);
                    let rand_entrance = xs_choice(xs, &room_data[rand_room].entrances);

                    write_out_room_entrance!(room_ptr, warp_ptr, rand_room, rand_entrance);
                }
            },
        }

        match &mut item_state {
            ItemState::None => {}
            ItemState::BadgeDeck(deck) => {
                let badges_set = get_badges_set();
                for item_ptr in room_data[name].items.iter() {
                    output.seek(SeekFrom::Start((room_ptr + item_ptr - room_base_ptr) as _))?;
                    let read_u32 = read_u32!();
                    if badges_set.contains(&read_u32) {
                        output.seek(SeekFrom::Current(-4))?;
                        if let Some(item) = deck.pop() {
                            println!("{:010x} -> {:010x}", read_u32, item);
                            output.write(&item.to_be_bytes())?;
                        }
                    }
                }
            }
            ItemState::TotalRandom => {
                for item_ptr in room_data[name].items.iter() {
                    output.seek(SeekFrom::Start((room_ptr + item_ptr - room_base_ptr) as _))?;
                    let rand_item = xs_u32(xs, 1, 0x16C);
                    let read_u32 = read_u32!();
                    if 0 < read_u32 && read_u32 < 0x200 {
                        output.seek(SeekFrom::Current(-4))?;
                        output.write(&rand_item.to_be_bytes())?;
                    }
                }
            }
        }
    }

    match room_mode {
        RoomMode::TotalRandom | RoomMode::None => {},
        RoomMode::StartWithHammer => {
            // start by the hammer by making "kmr_00" (Mario's fall area) lead there.
            write_out_room_entrance!(0x8ABF90, 2149846604, "kmr_04", 02);
            write_out_room_entrance!(0x8ABF90, 2149854336, "kmr_04", 02);
        },
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

fn get_badges_set() -> HashSet<u32> {
    // source: http://shrines.rpgclassics.com/n64/papermario/hacking.shtml
    // endian swapped from the above.
    vec![
        0x00E0, // Spin Smash
        0x00E1, // Multibounce
        0x00E2, // Power Plus
        0x00E3, // Dodge Master
        0x00E4, // Power Bounce
        0x00E5, // Spike Shield
        0x00E6, // First Attack
        0x00E7, // HP Plus
        0x00E8, // Quake Hammer
        0x00E9, // Double Dip
        0x00EB, // Sleep Stomp
        0x00EC, // Fire Shield
        0x00ED, // Quick Change
        0x00EE, // D-Down Pound
        0x00EF, // Dizzy Stomp
        0x00F1, // Pretty Lucky
        0x00F2, // Feeling Fine
        0x00F3, // Attack FX A
        0x00F4, // All or Nothing
        0x00F5, // HP Drain
        0x00F7, // Slow Go
        0x00F8, // FP Plus
        0x00F9, // Mega Rush
        0x00FA, // Ice Power
        0x00FB, // Defend Plus
        0x00FC, // Pay Off
        0x00FD, // Money Money
        0x00FE, // Chill Out
        0x00FF, // Happy Heart
        0x0100, // Zap Tap
        0x0102, // Right On!
        0x0103, // Runaway Pay
        0x0104, // Refund
        0x0105, // Flower Saver
        0x0106, // Triple Dip
        0x0107, // Hammer Throw
        0x0108, // Mega Quake
        0x0109, // Smash Charge
        0x010A, // Jump Charge
        0x010B, // S. Smash Chg.
        0x010C, // S. Jump Chg.
        0x010D, // Power Rush
        0x0111, // Last Stand
        0x0112, // Close Call
        0x0113, // P-Up, D-Down
        0x0114, // Lucky Day
        0x0116, // P-Down, D-Up
        0x0117, // Power Quake
        0x011A, // Heart Finder
        0x011B, // Flower Fiender
        0x011C, // Spin Attack
        0x011D, // Dizzy Attack
        0x011E, // I Spy
        0x011F, // Speedy Spin
        0x0120, // Bump Attack
        0x0121, // Power Jump
        0x0123, // Mega Jump
        0x0124, // Power Smash
        0x0126, // Mega Smash
        0x0127, // Power Smash
        0x0128, // Power Smash
        0x0129, // Deep Focus
        0x012B, // Shrink Smash
        0x012E, // D-Down Jump
        0x012F, // Shrink Stomp
        0x0130, // Damage Dodge
        0x0132, // Deep Focus
        0x0133, // Deep Focus
        0x0134, // HP Plus
        0x0135, // FP Plus
        0x0136, // Happy Heart
        0x0137, // Happy Heart
        0x0138, // Flower Saver
        0x0139, // Flower Saver
        0x013A, // Damage Dodge
        0x013B, // Damage Dodge
        0x013C, // Power Plus
        0x013D, // Power Plus
        0x013E, // Defend Plus
        0x013F, // Defend Plus
        0x0140, // Happy Flower
        0x0141, // Happy Flower
        0x0142, // Happy Flower
        0x0143, // Group Focus
        0x0144, // Peekaboo
        0x0145, // Attack FX D
        0x0146, // Attack FX B
        0x0147, // Attack FX E
        0x0148, // Attack FX C
        0x0149, // Attack FX F
        0x014A, // HP Plus
        0x014B, // HP Plus
        0x014C, // HP Plus
        0x014D, // FP Plus
        0x014E, // FP Plus
        0x014F, // FP Plus
        0x0151, // Attack FX F
        0x0152, // Attack FX F
        0x0153, // Attack FX F
    ].into_iter().collect()
}

/// the set from `get_badges_set` includes some badges that were not used in the game.
/// This contains only the used ones, and duplicates where there were ones in the game.
fn get_used_badges() -> Vec<u32> {
    vec![
        0x00ed,
        0x00ed,
        0x012f,
        0x0124,
        0x0120,
        0x0114,
        0x0121,
        0x0112,
        0x0107,
        0x00e8,
        0x013a,
        0x0103,
        0x011c,
        0x0148,
        0x00e5,
        0x00f7,
        0x0104,
        0x0135,
        0x0109,
        0x00e4,
        0x0134,
        0x011d,
        0x0146,
        0x0146,
        0x0133,
        0x0133,
        0x010d,
        0x0129,
        0x0111,
        0x0117,
        0x0136,
        0x00e7,
        0x00f8,
        0x012e,
        0x00f9,
        0x00ec,
        0x00ef,
        0x0141,
        0x0126,
        0x010c,
        0x0138,
        0x0138,
        0x0147,
        0x0123,
        0x0116,
        0x0113,
        0x0106,
        0x00fb,
        0x00fa,
        0x0132,
        0x013c,
    ]
}
