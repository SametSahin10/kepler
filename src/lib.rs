#[macro_use]
extern crate rocket;
#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate tokio;

use anyhow::Result;
use rocket::{fairing::AdHoc, figment::Figment, http::Header, Build, Rocket};

pub mod auth;
pub mod cas;
pub mod codec;
pub mod config;
pub mod ipfs;
pub mod orbit;
pub mod routes;
pub mod tz;

use orbit::load_orbits;
use routes::{
    batch_put_content, batch_put_create, cors, delete_content, get_content, list_content,
    put_content, put_create,
};
use tz::TezosBasicAuthorization;

pub async fn app(config: &Figment) -> Result<Rocket<Build>> {
    let kepler_config = config.extract::<config::Config>()?;

    // ensure KEPLER_DATABASE_PATH exists
    if !kepler_config.database.path.is_dir() {
        return Err(anyhow!(
            "KEPLER_DATABASE_PATH does not exist or is not a directory: {}",
            kepler_config.database.path.to_str().unwrap()
        ));
    }

    Ok(rocket::custom(config)
        .manage(load_orbits(kepler_config.database.path).await?)
        .manage(TezosBasicAuthorization)
        .mount(
            "/",
            routes![
                list_content,
                get_content,
                put_content,
                batch_put_content,
                delete_content,
                put_create,
                batch_put_create,
                cors
            ],
        )
        .attach(AdHoc::on_response("CORS", |_, resp| {
            Box::pin(async move {
                resp.set_header(Header::new("Access-Control-Allow-Origin", "*"));
                resp.set_header(Header::new(
                    "Access-Control-Allow-Methods",
                    "POST, GET, OPTIONS, DELETE",
                ));
                resp.set_header(Header::new("Access-Control-Allow-Headers", "*"));
                resp.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
            })
        })))
}

#[test]
#[should_panic]
async fn test_form() {
    use rocket::{http::ContentType, local::asynchronous::Client};

    #[post("/", format = "multipart/form-data", data = "<form>")]
    async fn stub_batch(form: Form<Vec<PutContent>>) {
        let content1 = &form.get(0).unwrap().content.value;
        let content2 = &form.get(1).unwrap().content.value;
        let p1 = r#"{"dummy":"obj"}"#;
        let p2 = r#"{"amother":"obj"}"#;
        assert_eq!(&content1, &p1.as_bytes());
        assert_eq!(&content2, &p2.as_bytes());
    }

    let form = r#"
-----------------------------28081028282221432566755324225
Content-Disposition: form-data; name="zyop8PQypg8QWqGNG92jJacYtEa56Mnaf9tLxDadXc8kPPxNVWZye"; filename="blob"
Content-Type: application/json

{"dummy":"obj"}
-----------------------------28081028282221432566755324225
Content-Disposition: form-data; name="zyop8PQypZnwFc58SPAxZTSCuG6R13jWSxQp8iBGNmBuV3HsrVyLx"; filename="blob"
Content-Type: application/json

{"amother":"obj"}
-----------------------------28081028282221432566755324225--
"#;

    let client = Client::debug_with(rocket::routes![stub_batch])
        .await
        .unwrap();
    let res = client
        .post("/")
        .header(
            "multipart/form-data; boundary=-----------------------------28081028282221432566755324225"
                .parse::<ContentType>()
                .unwrap()
        )
        .body(&form)
        .dispatch()
        .await;

    assert!(res.status().class().is_success());
}