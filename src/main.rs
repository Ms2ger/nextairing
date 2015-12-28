extern crate hyper;
extern crate kuchiki;

use hyper::Client;
use kuchiki::Html;
use kuchiki::iter::NodeIterator;
use std::env::args;
use std::error::Error;
use std::fmt;

const SELECTOR: &'static str = "#episode-list li:not(.old)";

#[derive(Debug)]
struct DataNotFoundError(&'static str);

impl fmt::Display for DataNotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Missing data: {}", self.0)
    }
}

impl Error for DataNotFoundError {
    fn description(&self) -> &str {
        self.0
    }
}

struct Episode {
    series: String,
    number: String,
    title: String,
    countdown: String,
}

impl fmt::Display for Episode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}: \"{}\" ({})", self.series.trim(), self.number.trim(), self.title.trim(), self.countdown.trim())
    }
}

fn get_page(series: &str) -> Result<Option<Episode>, Box<Error>> {
    let client = Client::new();
    let url = format!("http://nextairing.com/tv-shows/{}", series);
    let mut response = try!(client.get(&url).send());

    // Parse the html page
    let html = try!(Html::from_stream(&mut response));
    let document = html.parse();

    let item = match document.select(SELECTOR).unwrap().last() {
        Some(item) => item,
        None => return Ok(None),
    };

    let mut descendants = item.as_node().children().elements();
    let series = try!(descendants.next().ok_or(DataNotFoundError("series"))).text_contents();
    let number = try!(descendants.next().ok_or(DataNotFoundError("number"))).text_contents();
    let title = try!(descendants.next().ok_or(DataNotFoundError("title"))).text_contents();
    let countdown = try!(descendants.next().ok_or(DataNotFoundError("countdown"))).text_contents();
    Ok(Some(Episode {
        series: series,
        number: number,
        title: title,
        countdown: countdown,
    }))
}

fn get_airings<'a, I>(iterator: I) -> Result<Vec<String>, Box<Error>>
    where I: Iterator<Item=String>
{
    iterator.map(|series| {
        Ok(match try!(get_page(&series)) {
            Some(episode) => format!("{}", episode),
            None => format!("{}: no episode scheduled to air", series),
        })
    }).collect()
}

fn main() {
    match get_airings(args().skip(1)) {
        Ok(airings) => {
            for airing in airings {
                println!("{}", airing);
            }
        },
        Err(e) => println!("failure {:?}", e),
    }
}
