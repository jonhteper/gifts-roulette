use crate::roulette::{Participant, Roulette};
use calamine::{open_workbook, RangeDeserializerBuilder, Reader, Xlsx};
use delay_timer::anyhow::Context;
use eyre::{eyre, ContextCompat, Result, WrapErr};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

/// obtain the first sheet name in workbook
fn identify_sheet_name(file: &Xlsx<BufReader<File>>) -> Result<String> {
    let names = file.sheet_names();
    let name = ContextCompat::context(names.get(0), "No sheets in workbook")?;
    Ok(name.clone())
}

/// create a vector with participants data
/// # WARNING
/// The excel file needs have the next columns format:
///
/// `| email | name | info |`
pub fn read_excel(path: PathBuf) -> Result<Vec<Participant>> {
    let mut participants: Vec<Participant> = vec![];

    let mut wb: Xlsx<_> = open_workbook(&path).wrap_err("Error opening excel file")?;

    let sheet_name = identify_sheet_name(&wb).wrap_err("Error finding the sheet name")?;

    let range = ContextCompat::context(
        wb.worksheet_range(&sheet_name),
        format!("Error to access sheet {}", &sheet_name),
    )?
    .wrap_err("Error reading sheet")?;

    let mut iter = RangeDeserializerBuilder::new()
        .from_range(&range)
        .wrap_err("Error building iter")?;

    loop {
        match iter.next() {
            None => break,
            Some(row) => {
                let (email, name, info): (String, String, String) =
                    row.wrap_err("Error reading row data")?;

                participants.push(Participant {
                    name: name.trim().to_string(),
                    email: email.trim().to_string(),
                    info: info.trim().to_string(),
                });
            }
        }
    }

    Ok(participants)
}
