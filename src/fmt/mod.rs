use super::NbtFile;
use crate::nbt;
use std::fmt::Debug;

impl Debug for NbtFile {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for nbttag in self.nbtdata.nbt_tags() {
            for _ in 0..nbttag.position().depth() {
                write!(f, "\t")?;
            }

            let nbttag_value = nbttag.value();
            let tag_name = nbttag.name();

            match nbttag_value {
                nbt::NbtTagType::End(_) => write!(f, "End - {}", tag_name)?,
                nbt::NbtTagType::Byte(x) => write!(f, "{}[Byte]: {}", tag_name, x)?,
                nbt::NbtTagType::Short(x) => write!(f, "{}[Short]: {}", tag_name, x)?,
                nbt::NbtTagType::Int(x) => write!(f, "{}[Int]: {}", tag_name, x)?,
                nbt::NbtTagType::Long(x) => write!(f, "{}[Long]: {}", tag_name, x)?,
                nbt::NbtTagType::Float(x) => write!(f, "{}[Float]: {}", tag_name, x)?,
                nbt::NbtTagType::Double(x) => write!(f, "{}[Double]: {}", tag_name, x)?,
                nbt::NbtTagType::ByteArray(_) => {
                    write!(f, "{}[ByteArray]: [Values... see dump]", tag_name)?
                }
                nbt::NbtTagType::String(x) => write!(f, "{}[String]: {:?}", tag_name, x)?,
                nbt::NbtTagType::List(x) => write!(f, "{}[List]: {:?}", tag_name, x)?,
                nbt::NbtTagType::Compound(_) => write!(f, "{}[Compound]: ", tag_name)?,
                nbt::NbtTagType::IntArray(_) => {
                    write!(f, "{}[IntArray]: [Values... see dump]", tag_name)?
                }
                nbt::NbtTagType::LongArray(_) => {
                    write!(f, "{}[LongArray]: [Values... see dump]", tag_name)?
                }
            }
            writeln!(f)?;

            formatted_raw_values(f, nbttag, self.as_raw_bytes())?;

            writeln!(f)?;
        }

        Ok(())
    }
}

fn formatted_raw_values(
    f: &mut std::fmt::Formatter,
    nbttag: &nbt::NbtTag,
    rawbytes: &Vec<u8>,
) -> std::fmt::Result {
    for _ in 0..nbttag.position().depth() {
        write!(f, "\t")?;
    }
    write!(f, "Raw Bytes: ")?;
    write!(
        f,
        "ID[{}:{}] ",
        match nbttag.position().byte_start_id() {
            Some(x) => x.to_string(),
            None => "N/A".to_string(),
        },
        match nbttag.position().byte_end_id() {
            Some(x) => x.to_string(),
            None => "N/A".to_string(),
        }
    )?;
    write!(
        f,
        "Name[{}:{}] ",
        match nbttag.position().byte_start_name() {
            Some(x) => x.to_string(),
            None => "N/A".to_string(),
        },
        match nbttag.position().byte_end_name() {
            Some(x) => x.to_string(),
            None => "N/A".to_string(),
        }
    )?;

    let mut byte_start = 0;
    let mut byte_end = 0;
    let byte_start_dump;
    let byte_end_dump;

    match nbttag.value() {
        nbt::NbtTagType::Compound(_x) => {
            if let Some(x) = nbttag.position().byte_start_value() {
                byte_start = x;
            }
            byte_end = nbttag.position().byte_end_all_with_children();
            byte_start_dump = nbttag.position().byte_start_all();
            byte_end_dump = nbttag.position().byte_end_all();

            write!(f, "Value[{}:{}]", byte_start, byte_end)?;
        }
        nbt::NbtTagType::List(_x) => {
            if let Some(x) = nbttag.position().byte_start_value() {
                byte_start = x;
            }
            byte_end = nbttag.position().byte_end_all_with_children();
            byte_start_dump = nbttag.position().byte_start_all();
            byte_end_dump = nbttag.position().byte_end_all();

            write!(f, "Value[{}:{}]", byte_start, byte_end)?;
        }
        nbt::NbtTagType::End(_x) => {
            byte_start_dump = nbttag.position().byte_start_all();
            byte_end_dump = nbttag.position().byte_end_all();

            write!(f, "Value[{}]", "N/A")?;
        }
        _ => {
            if let Some(x) = nbttag.position().byte_start_value() {
                byte_start = x;
            }

            if let Some(x) = nbttag.position().byte_end_value() {
                byte_end = x;
            }

            byte_start_dump = nbttag.position().byte_start_all();
            byte_end_dump = nbttag.position().byte_end_all();

            write!(f, "Value[{}:{}]", byte_start, byte_end)?;
        }
    }

    let dump_hex = &rawbytes[byte_start_dump..byte_end_dump];

    writeln!(f)?;
    for _ in 0..nbttag.position().depth() {
        write!(f, "\t")?;
    }
    write!(f, "Hex Dump[{}:{}]", byte_start_dump, byte_end_dump)?;
    writeln!(f)?;
    formatted_raw_bytes(f, dump_hex, nbttag.position().depth())?;

    Ok(())
}

fn formatted_raw_bytes(
    f: &mut std::fmt::Formatter,
    rawbytes: &[u8],
    depth: i64,
) -> std::fmt::Result {
    for _ in 0..depth {
        write!(f, "\t")?;
    }

    for i in 0..rawbytes.len() {
        let byte = rawbytes[i];
        // Print a space every 4 bytes for grouping
        if i % 4 == 0 && i % 32 != 0 {
            write!(f, " ")?;
        }
        // Print a new line every 32 bytes
        if i % 32 == 0 && i != 0 {
            writeln!(f)?;
            for _ in 0..depth {
                write!(f, "\t")?;
            }
        }
        // Print the byte as hex
        write!(f, "{:02X} ", byte)?;
    }
    // Print a final new line
    write!(f, "\n")?;

    Ok(())
}
