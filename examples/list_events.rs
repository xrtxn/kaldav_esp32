use clap::Parser;

#[derive(Parser)]
struct Opt {
    #[arg(long)]
    username: Option<String>,
    #[arg(long)]
    password: Option<String>,
    url: String,
}

fn main() -> caldav::Result {
    env_logger::init();

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

        let events = calendar.events()?;

        if events.len() == 0 {
            println!("  no events");
            continue;
        }

        for event in &events[0..5] {
            let attr = match event.attr() {
                Ok(attr) => attr,
                Err(_) => continue,
            };

            if let Some(attr_event) = attr.events.get(0) {
                if let Some(property) = attr_event
                    .properties
                    .iter()
                    .filter(|x| x.name == "DTSTART")
                    .next()
                {
                    print!("  {} - ", property.value.clone().unwrap_or_default());
                }

                if let Some(property) = attr_event
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
