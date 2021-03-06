extern crate keyring;

use librespot_core::authentication::Credentials as RespotCredentials;
use librespot_protocol::authentication::AuthenticationType;
use configstore::{Configstore, AppUI};

use std::io::{self, Read};
use std::path::Path;

pub fn try_credentials() -> Result<RespotCredentials, String> {
    let config_store = Configstore::new("com.github.kelmes.gtkspot", AppUI::Graphical).unwrap();

    // defaults to: `~/.config/configstore-rs/com.github.kelmes.gtkspot/username.json`
    let username: String = match config_store.get("username") {
        Ok(x) => x,
        Err(e) => {
            println!("error retrieving username: {}", e);

            return(Err("no username found".to_string()));
        },
    };

    let service = "com.github.kelmes.gtkspot";

    let keyring = keyring::Keyring::new(&service, &username);

    let password = match keyring.get_password() {
        Ok(x) => x,
        Err(e) => {
            println!("error retrieving password: {}", e);
            return(Err("no password found".to_string()));
        }
    };

    if username.len() == 0 {
        return Err("empty username".to_string());
    }
    if password.len() == 0 {
        return Err("empty password".to_string());
    }

    let username = String::from(username);
    let auth_data = String::from(password).as_bytes().to_vec();
    Ok(RespotCredentials {
        username,
        auth_type: AuthenticationType::AUTHENTICATION_USER_PASS,
        auth_data,
    })
}

pub fn create_credentials(username: String, password: String) -> Result<RespotCredentials, String> {
    println!("creating credentials");
    let config_store = Configstore::new("com.github.kelmes.gtkspot", AppUI::Graphical).unwrap();

    let service = "com.github.kelmes.gtkspot";

    let keyring = keyring::Keyring::new(&service, &username);

    let auth_data = password.as_bytes().to_vec();
    match config_store.set("username", username.clone()) {
        Ok(x) => x,
        Err(e) => {println!("error storing username {}", e);}
    };

    match keyring.set_password(&password) {
        Ok(x) => {},
        Err(x) => {println!("error storing password: {}", x)},
    };
    let username = String::from(username);
    let auth_data = String::from(password).as_bytes().to_vec();

    Ok(RespotCredentials {
        username,
        auth_type: AuthenticationType::AUTHENTICATION_USER_PASS,
        auth_data,
    })
}

// TODO: better with futures?
fn auth_poller(url: &str) {
    //let app_sink = app_sink.clone();
    let url = url.to_string();
    std::thread::spawn(move || {
        let timeout = std::time::Duration::from_secs(5 * 60);
        let start_time = std::time::SystemTime::now();
        while std::time::SystemTime::now()
            .duration_since(start_time)
            .unwrap_or(timeout)
            < timeout
        {
            if let Ok(mut response) = reqwest::get(&url) {
                if response.status() != reqwest::StatusCode::ACCEPTED {
                    let result = match response.status() {
                        reqwest::StatusCode::OK => {
                            let creds = response
                                .json::<AuthResponse>()
                                .expect("Unable to parse")
                                .credentials;
                            Ok(creds)
                        }

                        _ => Err(format!(
                            "Facebook auth failed with code {}: {}",
                            response.status(),
                            response.text().unwrap()
                        )),
                    };
                    //app_sink
                    //    .send(Box::new(|s: &mut Cursive| {
                    //        s.set_user_data(result);
                    //        s.quit();
                    //    }))
                    //    .unwrap();
                    return;
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(1000));
        }

        //app_sink
        //    .send(Box::new(|s: &mut Cursive| {
        //        s.set_user_data::<Result<RespotCredentials, String>>(Err(
        //            "Timed out authenticating".to_string(),
        //        ));
        //        s.quit();
        //    }))
        //    .unwrap();
    });
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthResponse {
    pub credentials: RespotCredentials,
    pub error: Option<String>,
}
