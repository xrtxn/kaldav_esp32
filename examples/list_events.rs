use caldav::Requestable;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Opt {
    #[structopt(long)]
    username: Option<String>,
    #[structopt(long)]
    password: Option<String>,
    url: String,
}

fn main() -> caldav::result::Result<()>
{
    let opt = Opt::from_args();

    let mut client = caldav::client::Client::new(opt.url);

    if let Some(username) = opt.username {
        client.set_auth(Some(caldav::Authorization {
            username,
            password: opt.password,
        }));
    }

    let calendars = client.calendars()?;

    for (name, calendar) in calendars {
        println!("Using calendar '{}'", name);

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

            println!("  {:?}", attr);
        }
    }

    Ok(())
}
