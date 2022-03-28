use caldav::Requestable;
use clap::Parser;

#[derive(Parser)]
struct Opt {
    #[clap(long)]
    username: Option<String>,
    #[clap(long)]
    password: Option<String>,
    url: String,
}

fn main() -> caldav::Result {
    let opt = Opt::parse();

    let mut client = caldav::Client::new(opt.url);

    if let Some(username) = opt.username {
        client.set_auth(Some(caldav::Authorization {
            username,
            password: opt.password,
        }));
    }

    let calendars = client.calendars()?;

    for (name, calendar) in calendars {
        println!("Calendar '{}'", name);

        let calendars = calendar.events()?;

        if calendars.len() == 0 {
            println!("  no events");
            continue;
        }

        for calendar in &calendars[0..5] {
            let attr = match calendar.attr() {
                Ok(attr) => attr,
                Err(_) => continue,
            };

            if let Some(event) = attr.events.get(0) {
                if let Some(property) = event
                    .properties
                    .iter()
                    .filter(|x| x.name == "DTSTART")
                    .next()
                {
                    print!("  {} - ", property.value.clone().unwrap_or_default());
                }

                if let Some(property) = event
                    .properties
                    .iter()
                    .filter(|x| x.name == "SUMMARY")
                    .next()
                {
                    println!("{}", property.value.clone().unwrap_or_default());
                }
            }
        }
    }

    Ok(())
}
