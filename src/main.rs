use actix_files::Files;
use actix_web::{web, App, HttpServer, HttpResponse, middleware::Logger};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use env_logger::Env;

#[allow(unused_imports)]
mod monster_generated;
use monster_generated::my_game::sample::{Weapon, WeaponArgs, Monster, MonsterArgs, Vec3, Equipment, Color};

async fn index() -> HttpResponse {
    let mut builder = flatbuffers::FlatBufferBuilder::new_with_capacity(1024);

    let weapon_one_name = builder.create_string("Sword");
    let weapon_two_name = builder.create_string("Axe");

    // Use the `Weapon::create` shortcut to create Weapons with named field arguments.
    let sword = Weapon::create(&mut builder, &WeaponArgs{
        name: Some(weapon_one_name),
        damage: 3,
    });
    
    let axe = Weapon::create(&mut builder, &WeaponArgs{
        name: Some(weapon_two_name),
        damage: 5,
    });

    let weapons = builder.create_vector(&[sword, axe]);    
    let name = builder.create_string("Orc");
    let inventory = builder.create_vector(&[0u8, 1, 2, 3, 4, 5, 6, 7, 8, 9]);

    // Create the path vector of Vec3 objects.
    // Note that, for convenience, it is also valid to create a vector of
    // references to structs, like this:
    // let path = builder.create_vector(&[&x, &y]);
    let x = Vec3::new(1.0, 2.0, 3.0);
    let y = Vec3::new(4.0, 5.0, 6.0);
    let path = builder.create_vector(&[x, y]);

    // Create the monster using the `Monster::create` helper function. This
    // function accepts a `MonsterArgs` struct, which supplies all of the data
    // needed to build a `Monster`. To supply empty/default fields, just use the
    // Rust built-in `Default::default()` function, as demonstrated below.
    let orc = Monster::create(&mut builder, &MonsterArgs{
        pos: Some(&Vec3::new(1.0f32, 2.0f32, 3.0f32)),
        mana: 150,
        hp: 80,
        name: Some(name),
        inventory: Some(inventory),
        color: Color::Red,
        weapons: Some(weapons),
        equipped_type: Equipment::Weapon,
        equipped: Some(axe.as_union_value()),
        path: Some(path),
        ..Default::default()
    });

    // Call `finish()` to instruct the builder that this monster is complete.
    builder.finish(orc, None);    
    HttpResponse::Ok().body(builder.finished_data().to_owned())
}    

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    env_logger::from_env(Env::default().default_filter_or("info")).init();

    // create a self-signed temporary cert for testing:
    // openssl req -x509 -newkey rsa:4096 -nodes -keyout key.pem -out cert.pem -days 365 -subj "/C=CN/ST=sh/L=sh/O=stellar/OU=IT/CN=localhost"
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder.set_private_key_file("key.pem", SslFiletype::PEM).unwrap();
    builder.set_certificate_chain_file("cert.pem").unwrap();

    HttpServer::new(|| { 
            App::new()
                .wrap(Logger::default())
                .service(Files::new("/static", "./public").show_files_listing().use_last_modified(true))
                .service(web::scope("/users").route("/show", web::get().to(index)))
        })
        .bind_openssl("127.0.0.1:8088", builder)?
        .run()
        .await
}