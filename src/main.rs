use std::{
    error::Error,
    io::{BufReader, Cursor},
    ops::Deref,
};

use clap::Parser;
use ical::parser::ical::component::IcalCalendar;
use minicaldav::{ical::Ical, Credentials, Event};
use ureq::Agent;
use url::Url;

#[derive(Debug, Parser)]
struct Args {
    #[arg(short, long)]
    server: Url,
    #[arg(short, long)]
    username: String,
    #[arg(short, long)]
    password: String,
    #[arg(short, long)]
    calendar: String,
    #[arg(short, long)]
    ecampus_server: Url,
}

pub fn main() {
    let args = Args::parse();
    let agent = Agent::new();
    let username = args.username;
    let password = args.password;
    let credentials = Credentials::Basic(username.to_string(), password.to_string());
    let calendars = minicaldav::get_calendars(agent.clone(), &credentials, &args.server)
        .expect("Unable to fetch calendars");

    let relevant_calendar = calendars
        .into_iter()
        .find(|v| v.name() == &args.calendar)
        .unwrap_or_else(|| {
            panic!(
                "Your calendar was not found. Received a list of calendars but {} was not in it",
                args.calendar
            )
        });

    let cal =
        get_ecampus_calendar(&args.ecampus_server).expect("Unable to get calendar from ecampus");

    cal.events.iter().for_each(|v| {
        println!("-------\n{:?}", v.properties);
        minicaldav::save_event(
            agent.clone(),
            &credentials,
            Event::new(
                None,
                args.server.clone(),
                Ical {
                    name: "Test??".to_string(),
                    properties: <std::vec::Vec<ical::property::Property> as Clone>::clone(
                        &v.properties,
                    )
                    .into_iter()
                    .map(|v| minicaldav::ical::Property::new(&v.name, &v.value.unwrap_or_default()))
                    .collect(),
                    children: vec![],
                },
            ),
        )
        .unwrap();
    });
}

fn get_ecampus_calendar(ecampus_url: &Url) -> Result<IcalCalendar, Box<dyn Error>> {
    let reader = ureq::Agent::new()
        .get(ecampus_url.as_ref())
        .send(Cursor::new(vec![]))?
        .into_reader();

    let mut ical = ical::IcalParser::new(BufReader::new(reader));
    match ical.next() {
        Some(v) => Ok(v?),
        None => Err(Box::new(CustomError("Empty eCampus calendar".to_string()))),
    }
}

#[derive(Debug)]
struct CustomError(String);

impl std::error::Error for CustomError {}

impl std::fmt::Display for CustomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Custom ecampus sync error: {}", self.0)
    }
}
