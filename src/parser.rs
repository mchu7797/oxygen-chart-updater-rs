use encoding_rs::EUC_KR;
use std::io::{Read, Seek};
use std::str::Utf8Error;
use std::{fs, io};

// Constants
const OJN_SIGNATURE: &str = "ojn\0";
const HEADER_SIZE: usize = 300;
const OFFSET_TABLE_POSITION: u64 = 284;
const CHANNEL_MIN: i16 = 2;
const CHANNEL_MAX: i16 = 8;

// Error Types
#[derive(Debug)]
pub enum ParsingError {
    StringConversionError(Utf8Error),
    InvalidSignature,
    FileError(io::Error),
}

impl std::fmt::Display for ParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ParsingError::StringConversionError(e) => {
                write!(f, "Error caused when convert string: {}", e)
            }
            ParsingError::InvalidSignature => {
                write!(f, "Error caused when open file: invalid signature")
            }
            ParsingError::FileError(e) => write!(f, "Error caused when read file: {}", e),
        }
    }
}

impl From<Utf8Error> for ParsingError {
    fn from(err: Utf8Error) -> Self {
        ParsingError::StringConversionError(err)
    }
}

// Data Structures
#[derive(Debug)]
pub struct ChartInfo {
    pub chart_id: i32,
    pub title: String,
    pub artist: String,
    pub chart_maker: String,
    pub bpm: f32,
    pub level: [i16; 3],
    pub note_count: [i32; 3],
}

struct HeaderInfo {
    chart_id: i32,
    title: String,
    artist: String,
    chart_maker: String,
    bpm: f32,
    level: [i16; 3],
}

struct ChartOffsets {
    start: i32,
    end: i32,
}

// Implementation
impl ChartInfo {
    pub fn to_string(&self) -> String {
        format!(
            "{} : [{}, {}, {}, {}, [{}, {}, {}], [{}, {}, {}]]",
            self.chart_id,
            self.title,
            self.artist,
            self.chart_maker,
            self.bpm,
            self.level[0],
            self.level[1],
            self.level[2],
            self.note_count[0],
            self.note_count[1],
            self.note_count[2],
        )
    }
}

// Public Functions
pub fn parse_chart_info(file_path: &str) -> Result<ChartInfo, ParsingError> {
    let mut file = fs::File::open(file_path).map_err(ParsingError::FileError)?;
    let header = read_header(&mut file)?;
    let note_count = parse_exact_note_count(&mut file)?;

    Ok(ChartInfo {
        chart_id: header.chart_id,
        title: header.title,
        artist: header.artist,
        chart_maker: header.chart_maker,
        bpm: header.bpm,
        level: header.level,
        note_count,
    })
}

// Private Functions - Header Processing
fn read_header(file: &mut fs::File) -> Result<HeaderInfo, ParsingError> {
    let mut header_binary = [0; HEADER_SIZE];
    file.read_exact(&mut header_binary)
        .map_err(ParsingError::FileError)?;

    if std::str::from_utf8(&header_binary[4..8])? != OJN_SIGNATURE {
        return Err(ParsingError::InvalidSignature);
    }

    Ok(HeaderInfo {
        chart_id: i32::from_le_bytes(header_binary[0..4].try_into().unwrap()),
        bpm: f32::from_le_bytes(header_binary[16..20].try_into().unwrap()),
        level: [
            i16::from_le_bytes(header_binary[20..22].try_into().unwrap()),
            i16::from_le_bytes(header_binary[22..24].try_into().unwrap()),
            i16::from_le_bytes(header_binary[24..26].try_into().unwrap()),
        ],
        title: decode_euc_kr(&header_binary[108..172]),
        artist: decode_euc_kr(&header_binary[172..204]),
        chart_maker: decode_euc_kr(&header_binary[204..236]),
    })
}

fn decode_euc_kr(data: &[u8]) -> String {
    let null_pos = data.iter().position(|&x| x == 0).unwrap_or(data.len());
    let trimmed_data = &data[..null_pos];
    EUC_KR.decode(trimmed_data).0.to_string()
}

// Private Functions - Chart Processing
fn read_chart_offsets(file: &mut fs::File) -> Result<Vec<ChartOffsets>, ParsingError> {
    let mut temp_data = [0; 16];
    file.seek(io::SeekFrom::Start(OFFSET_TABLE_POSITION))
        .map_err(ParsingError::FileError)?;
    file.read_exact(&mut temp_data)
        .map_err(ParsingError::FileError)?;

    let mut offsets = Vec::with_capacity(3);
    for i in 0..3 {
        offsets.push(ChartOffsets {
            start: i32::from_le_bytes(temp_data[4 * i..4 * (i + 1)].try_into().unwrap()),
            end: i32::from_le_bytes(temp_data[4 * (i + 1)..4 * (i + 2)].try_into().unwrap()),
        });
    }
    Ok(offsets)
}

fn parse_exact_note_count(file: &mut fs::File) -> Result<[i32; 3], ParsingError> {
    let mut note_count = [0; 3];
    let mut offsets = read_chart_offsets(file)?;

    if offsets[2].end == 0 {
        offsets[2].end = file.metadata().map_err(ParsingError::FileError)?.len() as i32;
    }

    for (i, offset) in offsets.iter().enumerate() {
        note_count[i] = count_notes_in_chart(file, offset)?;
    }

    if note_count[0] == 0 && note_count[1] == 0 {
        note_count[0] = note_count[2];
        note_count[1] = note_count[2];
    }

    Ok(note_count)
}

fn count_notes_in_chart(file: &mut fs::File, offset: &ChartOffsets) -> Result<i32, ParsingError> {
    let mut count = 0;
    let mut is_long_note_pressed = [false; 7];
    let chart_size = (offset.end - offset.start) as usize;
    let mut chart_data = vec![0; chart_size];

    file.seek(io::SeekFrom::Start(offset.start as u64))
        .map_err(ParsingError::FileError)?;
    file.read_exact(&mut chart_data)
        .map_err(ParsingError::FileError)?;

    let mut current_offset = 0;
    while current_offset < chart_size {
        let channel = i16::from_le_bytes(
            chart_data[current_offset + 4..current_offset + 6]
                .try_into()
                .unwrap(),
        );
        let event_length = i16::from_le_bytes(
            chart_data[current_offset + 6..current_offset + 8]
                .try_into()
                .unwrap(),
        );

        if (CHANNEL_MIN..=CHANNEL_MAX).contains(&channel) {
            count += process_channel_events(
                &chart_data[current_offset + 8..],
                event_length,
                &mut is_long_note_pressed[channel as usize - 2],
            );
        }

        current_offset = current_offset + 8 + (event_length * 4) as usize;
    }

    Ok(count)
}

fn process_channel_events(event_data: &[u8], event_length: i16, is_pressed: &mut bool) -> i32 {
    let mut count = 0;

    for j in 0..event_length {
        let offset = 4 * j as usize;
        let event_value = i16::from_le_bytes(event_data[offset..offset + 2].try_into().unwrap());
        let note_type = event_data[offset + 3];

        if event_value != 0 {
            count += 1;

            if *is_pressed {
                match note_type {
                    0 | 2 => count -= 1,
                    3 => *is_pressed = false,
                    _ => {}
                }
            } else {
                match note_type {
                    2 => *is_pressed = true,
                    3 => count -= 1,
                    _ => {}
                }
            }
        }
    }

    count
}