use chrono::NaiveDateTime;
use nom::bytes::complete::{tag, take};
use nom::combinator::map_res;
use nom::error::ErrorKind;
use nom::multi::many_till;
use nom::number::complete::{le_f64, le_i32, le_i64, le_u32, le_u64};
use nom::IResult;
use pyo3::prelude::*;
use std::collections::HashMap;
use std::str;

#[derive(Debug)]
pub enum TagType {
    Empty8,
    Bool8(bool),
    Int8(i64),
    BitSet64(u64),
    // Color8,
    Float8(f64),
    DateTime(NaiveDateTime),
    // Float8Array,
    AnsiString(String),
    // WideString,
    // BinaryBlob,
}

#[pyclass]
#[derive(Debug)]
pub struct PQTimeTaggedData {
    pub version: String,
    pub events: Vec<EventRecord>,
}

#[derive(Debug)]
pub struct TagHeader {
    pub ident: String,
    pub idx: i32,
    pub typ: TagType,
}

#[pyclass]
#[derive(Debug)]
pub struct EventRecord {
    pub special: u8,
    pub channel: u8,
    pub timetag: u64,
}

fn parse_tag_enum(input: &[u8]) -> IResult<&[u8], TagType> {
    let (input, typ) = le_u32(input)?;
    match typ {
        0xFFFF0008 => {
            let (input, _) = le_i64(input)?;
            Ok((input, TagType::Empty8))
        }
        0x00000008 => {
            let (input, tag_value) = le_i64(input)?;
            Ok((input, TagType::Bool8(tag_value != 0)))
        }
        0x10000008 => {
            let (input, tag_value) = le_i64(input)?;
            Ok((input, TagType::Int8(tag_value)))
        }
        0x11000008 => {
            let (input, tag_value) = le_u64(input)?;
            Ok((input, TagType::BitSet64(tag_value)))
        }
        0x12000008 => {
            let (_input, _tag_value) = le_f64(input)?;
            panic!("implement it later");
            // Ok((input, TagType::Color8))
        }
        0x20000008 => {
            let (input, tag_value) = le_f64(input)?;
            Ok((input, TagType::Float8(tag_value)))
        }
        0x21000008 => {
            let (input, tag_value) = le_f64(input)?;
            let time = NaiveDateTime::from_timestamp_opt(
                ((tag_value - 25569f64) * 86400f64).round() as i64,
                0,
            )
            .unwrap();
            Ok((input, TagType::DateTime(time)))
        }
        0x2001FFFF => {
            let (_input, _tag_value) = le_f64(input)?;
            panic!("implement it later");
            // Ok((input, TagType::Float8Array))
        }
        0x4001FFFF => {
            let (input, length) = le_u64(input)?;
            let (input, str) = map_res(take(length), str::from_utf8)(input)?;
            Ok((input, TagType::AnsiString(str.to_string())))
        }
        0x4002FFFF => {
            let (_input, _tag_value) = le_f64(input)?;
            panic!("implement it later");
            // Ok((input, TagType::WideString))
        }
        0xFFFFFFFF => {
            let (_input, _tag_value) = le_f64(input)?;
            panic!("implement it later");
            // Ok((input, TagType::BinaryBlob))
        }
        _ => Err(nom::Err::Error(nom::error::Error::new(
            input,
            ErrorKind::Not,
        ))),
    }
}

fn debug_type(tag: &TagType) -> String {
    match tag {
        TagType::AnsiString(s) => s.trim_end_matches('\0').to_string(),
        TagType::Int8(i) => i.to_string(),
        TagType::BitSet64(i) => format!("0x{:0>16x}", i),
        TagType::Float8(i) => format!("{:.6E}", i),
        TagType::Empty8 => "<empty Tag>".to_string(),
        TagType::Bool8(b) => (if *b { "True" } else { "False" }).to_string(),
        TagType::DateTime(t) => t.format("%a %b %d %H:%M:%S %Y").to_string(),
    }
}
fn parse_tag_header(input: &[u8]) -> IResult<&[u8], TagHeader> {
    let (input, ident) = parse_tag_ident(input)?;
    let (input, idx) = le_i32(input)?;
    let (input, typ) = parse_tag_enum(input)?;
    if idx > -1 {
        let s = format!("{}({})", ident, idx);
        println!("{: <40}{}", s, debug_type(&typ));
    } else {
        println!("{: <40}{}", ident, debug_type(&typ));
    }

    Ok((
        input,
        TagHeader {
            ident: ident.to_string(),
            idx,
            typ,
        },
    ))
}
fn parse_tag_ident(input: &[u8]) -> IResult<&[u8], &str> {
    let (input, ident) = map_res(take(32u8), str::from_utf8)(input)?;
    Ok((input, ident.trim_end_matches('\0')))
}
fn check_header_end(input: &[u8]) -> IResult<&[u8], ()> {
    let (input, ident) = parse_tag_ident(input)?;
    if ident == "Header_End" {
        let (input, _idx) = le_i32(input)?;
        let (input, _typ) = parse_tag_enum(input)?;
        println!("Header_End                              <empty Tag>\n-----------------------\nMultiHarp T2 data\n\nrecord# chan   nsync truetime/ps");
        Ok((input, ()))
    } else {
        Err(nom::Err::Error(nom::error::Error::new(
            input,
            ErrorKind::Not,
        )))
    }
}
fn parse_tag_headers(input: &[u8]) -> IResult<&[u8], Vec<TagHeader>> {
    let (input, (tag_headers, _)) = many_till(parse_tag_header, check_header_end)(input)?;
    Ok((input, tag_headers))
}

const _T2WRAPAROUND_V1: u64 = 33552000;
const T2WRAPAROUND_V2: u64 = 33554432;

fn parse_ht2_event_records(
    input: &[u8],
    num_records: usize,
    _global_resolution: f64,
) -> IResult<&[u8], Vec<EventRecord>> {
    let mut overflow_correction: u64 = 0;
    let mut acc = Vec::with_capacity(num_records);
    let mut i = input;
    loop {
        match le_u32(i) {
            Err(nom::Err::Error(_)) => return Ok((i, acc)),
            Err(e) => return Err(e),
            Ok((li, value)) => {
                let special = ((value >> 31) & 0x01) as u8;
                let channel = ((value >> 25) & 0x3F) as u8;
                let timetag = (value & 0x1FFFFFF) as u64;
                if special == 1 {
                    if channel == 0x3F {
                        if timetag == 0 {
                            overflow_correction = T2WRAPAROUND_V2;
                        } else {
                            overflow_correction += T2WRAPAROUND_V2 * timetag;
                        }
                    } else if (1..=15).contains(&channel) { // marker
                    }
                    if channel == 0 {
                        // sync
                        let truetime = overflow_correction + timetag;
                        acc.push(EventRecord {
                            special,
                            channel,
                            timetag: truetime,
                        })
                    }
                } else {
                    // photon
                    let truetime = overflow_correction + timetag;
                    acc.push(EventRecord {
                        special,
                        channel,
                        timetag: truetime,
                    })
                }
                i = li;
            }
        }
    }
}
pub fn parse_t2_ptu(input: &[u8]) -> IResult<&[u8], PQTimeTaggedData> {
    let (input, _) = tag(&b"PQTTTR\0\0"[..])(input)?;
    let (input, version) = map_res(take(8u8), str::from_utf8)(input)?;
    println!("Tag version: {}\n", version.trim_end_matches('\0'));
    let (input, tag_headers) = parse_tag_headers(input)?;
    let num_records = if let TagType::Int8(n) = &tag_headers
        .iter()
        .find(|rec| rec.ident == "TTResult_NumberOfRecords")
        .unwrap()
        .typ
    {
        n
    } else {
        &100
    };
    let global_resolution = if let TagType::Float8(n) = &tag_headers
        .iter()
        .find(|rec| rec.ident == "MeasDesc_GlobalResolution")
        .unwrap()
        .typ
    {
        n
    } else {
        &0f64
    };
    // let resolution = &tag_headers.iter().find(|rec| rec.ident == "MeasDesc_GlobalResolution").unwrap().typ;
    let (input, rec) = parse_ht2_event_records(input, *num_records as usize, *global_resolution)?;
    Ok((
        input,
        PQTimeTaggedData {
            version: version.to_string(),
            events: rec,
        },
    ))
}

#[pyclass]
pub struct PtuParser {
    events: HashMap<u8, Vec<u64>>, // contains truetime in global res
    overflow_correction: u64,
}

#[pymethods]
impl PtuParser {
    #[new]
    fn new() -> Self {
        let mut s = Self {
            events: HashMap::new(),
            overflow_correction: 0u64,
        };
        for ch in 0..65 {
            s.events.insert(ch, Vec::new());
        }
        s
    }

    fn parse_records(&mut self, records: Vec<u32>) {
        for record in records {
            self.parse_record(record);
        }
    }

    fn parse_record(&mut self, data: u32) {
        let special = (data >> 31) & 0x01;
        let mut channel = ((data >> 25) & 0x3F) as u8;
        let timetag = (data & 0x1FFFFFF) as u64;
        if special == 1 {
            if channel == 0x3F {
                if timetag == 0 {
                    self.overflow_correction += T2WRAPAROUND_V2;
                } else {
                    self.overflow_correction += T2WRAPAROUND_V2 * timetag;
                }
            }
            if channel == 0 {
                let truetime = self.overflow_correction + timetag;
                let v = self.events.get_mut(&channel).unwrap();
                v.push(truetime);
            }
        } else {
            channel += 1;
            let truetime = self.overflow_correction + timetag;
            let v = self.events.get_mut(&channel).unwrap();
            v.push(truetime);
        }
    }

    fn get_events(&self) -> HashMap<u8, Vec<u64>> {
        self.events.clone()
    }
    fn get_oflcorrection(&self) -> u64 {
        self.overflow_correction
    }
}
