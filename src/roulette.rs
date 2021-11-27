use crate::read_excel;
use crate::send::MailerClient;
use bcrypt::{hash, verify, DEFAULT_COST};
use eyre::{eyre, ContextCompat, Result, WrapErr};
use lettre_email::{Email, EmailBuilder};
use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufWriter, Read, Write};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Roulette {
    participants: Vec<Participant>,
    random: bool,
    output_file_path: PathBuf,
    saved: bool,
    couples: Couples,
}

impl Roulette {
    fn check_extension(path: &PathBuf) -> Result<()> {
        let extension_str = ContextCompat::context(
            ContextCompat::context(path.extension(), "Error reading extension")?.to_str(),
            "Error in to-str conversion",
        )?;

        if extension_str != "json" {
            return Err(eyre!("Bad extension, only .json files"));
        }

        Ok(())
    }

    /// Roulette constructor
    ///
    /// # Arguments
    ///
    /// * `participants`: Participant list.
    /// * `store_path`: path to json file where the sort result will saved.
    ///
    /// returns: Result<Roulette, Report>
    ///
    /// # Examples
    ///
    /// ```
    /// let participants: Vec<Participant> = vec![];
    /// let err_roulette = Roulette::from(participants, "dir/db.txt")?; // Error
    /// ```
    /// ```
    /// let participants: Vec<Participant> = vec![];
    /// let roulette = Roulette::from(participants, "dir/db.json")?;
    /// ```
    pub fn new(participants: Vec<Participant>, output_path: &str) -> Result<Self> {
        let path = PathBuf::from(&output_path);
        let _ = Roulette::check_extension(&PathBuf::from(&output_path))
            .wrap_err("Error with output file extension")?;

        Ok(Roulette {
            participants,
            random: false,
            output_file_path: path,
            saved: false,
            couples: Couples::new(),
        })
    }

    fn shuffle(&mut self) -> Result<()> {
        if self.random {
            return Ok(());
        }

        let mut rng = thread_rng();
        self.participants.shuffle(&mut rng);
        self.random = true;

        Ok(())
    }

    fn participants_to_couples(&mut self) -> Result<()> {
        if !self.random {
            return Err(eyre!("Participants not shuffled"));
        }

        for i in 0..self.participants.len() - 1 {
            let a = &self.participants[i];
            let b = &self.participants[i + 1];

            self.couples
                .couples
                .push(vec![a.name.clone(), b.name.clone()]);
        }

        // Create last couple
        let first =
            ContextCompat::context(self.participants.get(0), "Not participants in Roulette")?
                .clone()
                .name;
        let last = ContextCompat::context(
            self.participants.get(self.participants.len() - 1),
            "Not enough participants in Roulette",
        )?;

        self.couples.couples.push(vec![last.name.clone(), first]);

        Ok(())
    }

    pub fn get_couples(&self) -> Result<Couples> {
        let mut couples = Couples::new();

        for c in &self.couples.couples {
            let hashed = hash(c[1].clone(), DEFAULT_COST).wrap_err("Error creating hash")?;
            couples.couples.push(vec![c[0].clone(), hashed]);
        }

        Ok(couples)
    }

    fn save(&mut self) -> Result<()> {
        if self.saved {
            return Ok(());
        }

        let _ = self
            .participants_to_couples()
            .wrap_err("Error parsing couples")?;

        // rand again, before print json file
        self.couples.rand();

        let mut file = File::create(&self.output_file_path)
            .wrap_err(format!("Error opening file {:?}", &self.output_file_path))?;

        let data =
            serde_json::to_string_pretty(&self.get_couples().wrap_err("Error acceding couples")?)
                .wrap_err("Error serialize data")?;

        BufWriter::new(file)
            .write_all(data.as_bytes())
            .wrap_err("Error writing data in file")?;

        self.saved = true;
        Ok(())
    }

    /// shuffle participants and save sort in file
    pub fn run(&mut self) -> Result<()> {
        let _ = self.shuffle().wrap_err("Error with shuffle")?;

        let _ = self.save().wrap_err("Error saving data")?;

        Ok(())
    }

    fn get_participant(&self, name: &str) -> Option<Participant> {
        for p in &self.participants {
            if p.name == name {
                return Some(p.clone());
            }
        }

        None
    }

    fn create_email(
        client: &MailerClient,
        sender: Participant,
        benefited: Participant,
    ) -> Result<Email> {
        let mail = EmailBuilder::new()
            .to(sender.email)
            .from(client.get_user())
            .subject("Gift Exchange")
            .text(format!(
                "Your gift is for: {}\nContext:{}",
                benefited.name, benefited.info
            ))
            .build()
            .wrap_err("Error building email")?;

        Ok(mail)
    }

    pub fn send_emails(&self) -> Result<()> {
        let mut mail_client = MailerClient::new().wrap_err("Error creating Mailer Client")?;

        println!("Sending emails...");
        for couple in &self.couples.couples {
            let sender = ContextCompat::context(
                self.get_participant(&couple[0]),
                "Participant is not in list",
            )?;

            let benefited = ContextCompat::context(
                self.get_participant(&couple[1]),
                "Participant is not in list",
            )?;

            let email = Roulette::create_email(&mail_client, sender, benefited)
                .wrap_err("Error creating email")?;
            let _ = mail_client
                .send_mail(email.into())
                .wrap_err("Error sending email")?;
        }

        Ok(())
    }

    /// **WARNING**: this function uses a lot of computing power
    fn decrypt_couples(&self) -> Result<()> {
        Ok(())
    }

    /// Use only with sorted Participants
    pub fn from_files(input_path: &str, output_path: &str) -> Result<Self> {
        let output_file_path = PathBuf::from(&output_path);
        let _ = Roulette::check_extension(&output_file_path)
            .wrap_err("Error with output file extension")?;
        let participants =
            read_excel(PathBuf::from(&input_path)).wrap_err("Error reading excel file")?;

        let couples =
            Couples::from_file(&output_file_path).wrap_err("Error obtaining couples from file")?;

        Ok(Roulette {
            participants,
            random: true,
            output_file_path,
            saved: true,
            couples,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Participant {
    pub name: String,
    pub email: String,
    pub info: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Couples {
    pub couples: Vec<Vec<String>>,
}

impl Couples {
    pub fn new() -> Self {
        Couples { couples: vec![] }
    }

    pub fn rand(&mut self) {
        let mut rng = thread_rng();
        self.couples.shuffle(&mut rng);
    }

    fn parse_file(path: &PathBuf) -> Result<String> {
        let mut file = File::open(&path).wrap_err(format!("Error opening file: {:?}", &path))?;
        let mut buffer = String::new();
        let _ = file
            .read_to_string(&mut buffer)
            .wrap_err("Error reading file")?;
        Ok(buffer)
    }

    /// get couples from json file.
    ///
    /// **WARNING**: The couples will are hashed
    fn from_file(path: &PathBuf) -> Result<Self> {
        let data = Couples::parse_file(&path).wrap_err("Error in file-to-string conversion")?;
        let couples: Couples = serde_json::from_str(&data).wrap_err("Error parsing json data")?;

        Ok(couples)
    }
}
