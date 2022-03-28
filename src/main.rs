#[macro_use] extern crate rocket;

use std::path::Path;
use std::path::PathBuf;
use std::collections::HashMap;

use rocket::fs::NamedFile;
use rocket::fs::relative;

#[get("/")]
async fn index() -> Option<NamedFile> {
    let path = Path::new(relative!("/static/fox.png"));
    NamedFile::open(&path).await.ok()
}

#[get("/file/<file..>")]
async fn file(file: PathBuf) -> Option<NamedFile> {
    let mut path = Path::new(relative!("static/file")).join(file);
    if !path.exists() {
        path = Path::new("/drives/LinuxData0/aikufurr/Pictures/c2mec31oelg61.png").to_path_buf();
    }
    NamedFile::open(&path).await.ok()
}

#[get("/presence?<cmus_base>&<cmus_artist>&<cmus_album>")]
async fn presence(cmus_base: Option<&str>, cmus_artist: Option<&str>, cmus_album: Option<&str>) -> Option<NamedFile> {
    let mut base: String;
    match cmus_base {
        Some(arg) => base = arg.replace("%20", " "),
        None => base = "".to_owned()
    }
    let artist: String;
    match cmus_artist {
        Some(arg) => artist = arg.replace("%20", " "),
        None => artist = "".to_owned()
    }
    let album: String;
    match cmus_album {
        Some(arg) => album = arg.replace("%20", " "),
        None => album = "".to_owned()
    }

    let mut path: String = relative!("/static/fox.png").to_owned();

    if !artist.is_empty() && !album.is_empty() {
        if !base.is_empty() {
            base = base + "/";
        }
        let dir = "/home/aikufurr/Music/".to_owned() + &base + &artist + "/" + &album;
        if Path::new(&format!("{}/cover.png", dir)).exists() {
            path = format!("{}/cover.png", dir);
        } else if Path::new(&format!("{}/cover.jpg", dir)).exists() {
            path = format!("{}/cover.jpg", dir);
        } else if Path::new(&format!("{}/cover0.png", dir)).exists() {
            path = format!("{}/cover0.png", dir);
        } else if Path::new(&format!("{}/cover0.jpg", dir)).exists() {
            path = format!("{}/cover0.jpg", dir);
        } else if Path::new(&format!("{}/cover1.png", dir)).exists() {
            path = format!("{}/cover1.png", dir);
        } else if Path::new(&format!("{}/cove1r.jpg", dir)).exists() {
            path = format!("{}/cover1.jpg", dir);
        }
    }

    NamedFile::open(&path).await.ok()
}

#[get("/wordle")]
async fn wordle() -> Option<NamedFile> {
    let mut path = Path::new(relative!("static/wordle/wordle.html"));
    if !path.exists() {
        path = Path::new(relative!("/static/fox.png"));
    }
    NamedFile::open(&path).await.ok()
}

#[catch(404)]
async fn not_found() -> Option<NamedFile> {
    let path = Path::new("/drives/LinuxData0/aikufurr/Pictures/c2mec31oelg61.png").to_path_buf();
    NamedFile::open(&path).await.ok()
}

#[derive(Debug, FromForm, UriDisplayQuery)]
struct EmailWebhook {
    recipient: String,
    subject: String,
}

fn db_make_email(email: &EmailWebhook) -> String {
    let conn = sqlite::open("database.db").unwrap();

    let id: String = uuid::Uuid::new_v4().to_string().replace("-", "") + ".png";

    let mut statement = conn.prepare("INSERT INTO emails (ID, RECIPIENT, SUBJECT) VALUES (?,?,?)").unwrap();
    statement.bind(1, &*id).unwrap();
    statement.bind(2, &*email.recipient).unwrap();
    statement.bind(3, &*email.subject).unwrap();
    
    statement.next().unwrap();
    
    id
}

fn db_get_email(id: &str) -> EmailWebhook {
    let conn = sqlite::open("database.db").unwrap();

    let mut statement = conn.prepare("SELECT RECIPIENT, SUBJECT FROM emails WHERE ID = ?").unwrap();
    statement.bind(1, id).unwrap();

    let mut eml = EmailWebhook {
        recipient: "".to_owned(),
        subject: "".to_owned()
    };

    eml.recipient = "".to_owned();

    while let sqlite::State::Row = statement.next().unwrap() {
        eml.recipient = statement.read::<String>(0).unwrap();
        eml.subject = statement.read::<String>(1).unwrap();
    }

    eml
}

fn db_del_email(id: &str) {
    let conn = sqlite::open("database.db").unwrap();

    let mut statement = conn.prepare("DELETE FROM emails WHERE ID = ?").unwrap();
    statement.bind(1, id).unwrap();
    statement.next().unwrap();
}

#[get("/email?<email..>")]
async fn email_make(email: EmailWebhook) -> String {

    db_make_email(&email)
}

#[get("/email/<id>")]
async fn email(id: String) -> Option<NamedFile> {
    let details = db_get_email(&id);

    let mut map = HashMap::new();
    map.insert("content", "<@308681202548604938>, your email to `".to_owned() + &details.recipient + "` about `" + &details.subject + "` was read!");

    let client = reqwest::Client::new();
    let response = client.post("https://discord.com/api/webhooks/957769610244009996/XbnkWFQEjJ4xNWhTOrgo9Rq7Pl8JsEHQsTBgeHUdTYufa70L_m9b9B7VYPpKhKduuU6G")
        .json(&map)
        .send()
        .await;

    match response {
        Ok(_) => {},
        Err(_) => {}
    };

    db_del_email(&id);

    let path = Path::new(relative!("/static/1x1.png")).to_path_buf();
    NamedFile::open(&path).await.ok()
}

fn init_db() {
    let conn = sqlite::open("database.db").unwrap();

    let mut statement = conn.prepare("SELECT count(name) FROM sqlite_master WHERE type='table' AND name = ?").unwrap();
    statement.bind(1, "emails").unwrap();
    while let sqlite::State::Row = statement.next().unwrap() {
        if statement.read::<i64>(0).unwrap() == 0 {
            statement = conn.prepare(r#"CREATE TABLE "emails" (
                "ID"	TEXT NOT NULL UNIQUE,
                "RECIPIENT"	TEXT,
                "SUBJECT"	TEXT,
                PRIMARY KEY("ID")
            )"#).unwrap();
            statement.next().unwrap();
        }
        break;
    }
}

#[launch]
fn rocket() -> _ {
    init_db();
    rocket::build().mount("/", routes![index, file, presence, wordle, email_make, email]).register("/", catchers![not_found])
}
