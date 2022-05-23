use std::ops::Range;

use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use wiremock::matchers::{any, method, AnyMatcher};
use wiremock::{Mock, MockServer, ResponseTemplate, Times};
use zero2prod::configuration::{get_configuration, DatabaseSettings};
use zero2prod::startup::{get_connection_pool, Application};
use zero2prod::telemetry::{get_subscriber, init_subscriber};

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
    pub email_server: MockServer,
    pub port: u16,
}

impl TestApp {
    pub async fn post_subscription(&self, body: String) -> reqwest::Response {
        reqwest::Client::new()
            .post(&format!("{}/subscriptions", &self.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("FAiled to execute request in test APP")
    }

    pub async fn get_confirmation_links(
        &self,
        email_request: &wiremock::Request,
    ) -> ConfirmationLinks {
        let mut links = ConfirmationLinks::from_request(email_request);
        links.set_port(self.port);
        links
    }

    pub async fn post_newsletter(&self, body: serde_json::Value) -> reqwest::Response {
        reqwest::Client::new()
            .post(&format!("{}/newsletters", self.address))
            .json(&body)
            .send()
            .await
            .expect("request failed")
    }

    pub async fn mock_mail_server<T: Into<Times>>(&self, meth: &str, code: u16, exp: T) {
        let meth = match meth {
            "g" => "GET",
            "p" => "POST",
            _ => "GET",
        };
        // let exp: Times = exp.into();
        Mock::given(any())
            .and(method(meth))
            .respond_with(ResponseTemplate::new(code))
            .expect(exp.into())
            .mount(&self.email_server)
            .await
    }
}

#[derive(Debug)]
pub struct ConfirmationLinks {
    pub html: reqwest::Url,
    pub plain_text: reqwest::Url,
}

impl ConfirmationLinks {
    pub fn from_request(request: &wiremock::Request) -> Self {
        let body: serde_json::Value = serde_json::from_slice(&request.body).unwrap();
        let get_links = |s: &str| {
            let links = linkify::LinkFinder::new()
                .links(s)
                .filter(|x| *x.kind() == linkify::LinkKind::Url)
                .collect::<Vec<_>>();
            assert_eq!(links.len(), 1);
            links[0].as_str().to_owned()
        };
        let html = reqwest::Url::parse(&get_links(&body["HtmlBody"].as_str().unwrap())).unwrap();
        let plain_text =
            reqwest::Url::parse(&get_links(&body["TextBody"].as_str().unwrap())).unwrap();

        Self { html, plain_text }
    }
    pub fn set_port(&mut self, port: u16) {
        self.html.set_port(Some(port)).expect("Failed to set port");
        self.plain_text
            .set_port(Some(port))
            .expect("Failed to set port");
    }
}

static TRACING: Lazy<()> = Lazy::new(|| {
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber("test".into(), "debug".into(), std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber("test".into(), "debug".into(), std::io::sink);
        init_subscriber(subscriber);
    };
});

pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let email_server = MockServer::start().await;
    let configuration = {
        let mut c = get_configuration().expect("test init faild in get config");
        c.database.database_name = uuid::Uuid::new_v4().to_string();
        c.application.port = 0;
        c.email_client.timeout_ms = 50;
        c.email_client.base_url = email_server.uri();
        c
    };

    //create and midgrate database
    configure_database(&configuration.database).await;

    let application = Application::build(configuration.clone())
        .await
        .expect("DAuild to build Application");
    let application_port = application.port();
    let _ = tokio::spawn(application.run_until_stopped());

    TestApp {
        address: format!("http://localhost:{}", application_port),
        db_pool: get_connection_pool(&configuration.database),
        email_server,
        port: application_port,
    }
}

async fn configure_database(config: &DatabaseSettings) -> PgPool {
    PgConnection::connect_with(&config.without_db())
        .await
        .expect("Echec de connection à Postgre")
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Echec de création de la bdd de test");

    let connection_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Echec de création du pool");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Echec des migrations.");

    connection_pool
}

pub async fn mock_mail_server<G>(
    app: &TestApp,
    given: Option<G>,
    meth: Option<&str>,
    code: Option<u16>,
    expect: Option<u64>,
) where
    G: 'static + wiremock::Match,
{
    let meth = match meth {
        Some("p") => "POST",
        _ => "GET",
    };
    let code = code.unwrap_or(200);

    let mock = match given {
        Some(g) => Mock::given(g),
        None => Mock::given(any()),
    };

    let mock = mock
        .and(method(meth))
        .respond_with(ResponseTemplate::new(code));

    let mock = match expect {
        Some(x) => {
            let nb: Times = x.into();
            mock.expect(nb)
        }
        None => mock,
    };
    // let mock = match expect {
    //     x if x < 0 => mock,
    //     x => {
    //         let nb: u64 = x.try_into().unwrap();
    //         let times: Times = nb.into();
    //         mock.expect(times)
    //     }
    // };

    mock.mount(&app.email_server).await;
}

// #[macro_export]
// macro_rules! mockmail {
//     // par défaut: any get 200
//     // ($app:ident) => {
//     //     crate::helpers::mock_mail_server(&$app, None, None, None, None)
//     // }
//     // ($app:ident) => {};
//     ($app:ident  $(g=$given:expr)? , $(m=$m:literal)?) => {
//         let pp = wiremock::matchers::AnyMatcher;
//         $(let pp= &$given;)?
//         let method = wiremock::matchers::method("GET");
//         $(let method=wiremock::matchers::method($m);)?
//         dbg!(&pp, method);
//     };
//     ($app:ident,$meth:expr,$code:literal) => {
//         // ($app:ident,$meth:expr,$code:literal) => {
//         crate::helpers::mock_mail_server(
//             &$app,
//             Some(wiremock::matchers::AnyMatcher),
//             Some($meth),
//             Some($code),
//             None,
//         )
//         .await
//     };
// }
