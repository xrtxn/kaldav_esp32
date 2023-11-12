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

        let objects = calendar.events()?;

        if objects.is_empty() {
            println!("  no events");
            continue;
        }

        for object in objects.take(5) {
            for event in object.events {
                println!("  {} - {}", event.dtstart, event.summary.unwrap_or_default());
            }
        }
    }

    Ok(())
}
