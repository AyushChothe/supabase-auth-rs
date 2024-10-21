use core::time;
use std::{collections::HashMap, env, thread};

use supabase_auth::models::{
    AuthClient, LogoutScope, ResendParams, SignInWithOAuthOptions, SignInWithSSO,
    SignUpWithPasswordOptions, UpdateUserPayload,
};

fn create_test_client() -> AuthClient {
    AuthClient::new_from_env().unwrap()
}

#[tokio::test]
async fn create_client_test_valid() {
    let auth_client = AuthClient::new_from_env().unwrap();

    assert!(*auth_client.project_url() == env::var("SUPABASE_URL").unwrap())
}

#[tokio::test]
async fn test_login_with_email() {
    let auth_client = create_test_client();

    let demo_email = env::var("DEMO_EMAIL").unwrap();
    let demo_password = env::var("DEMO_PASSWORD").unwrap();

    let session = auth_client
        .login_with_email(&demo_email, &demo_password)
        .await
        .unwrap();

    assert!(session.user.email == demo_email)
}

#[tokio::test]
async fn test_login_with_email_invalid() {
    let auth_client = create_test_client();

    let demo_email = "invalid@demo.com";
    let demo_password = "invalid";

    let session = auth_client
        .login_with_email(demo_email, demo_password)
        .await;

    assert!(session.is_err())
}

#[tokio::test]
async fn sign_up_with_email_test_valid() {
    let auth_client = create_test_client();

    let uuid = uuid::Uuid::now_v7();

    let demo_email = format!("signup__{}@demo.com", uuid);
    let demo_password = "ciJUAojfZZYKfCxkiUWH";

    let session = auth_client
        .sign_up_with_email_and_password(demo_email.as_ref(), demo_password, None)
        .await
        .unwrap();

    // Wait to prevent running into Supabase rate limits when running cargo test
    let one_minute = time::Duration::from_secs(60);
    thread::sleep(one_minute);

    assert!(session.user.email == demo_email)
}

#[tokio::test]
async fn test_mobile_flow() {
    let auth_client = create_test_client();

    let demo_phone = env::var("DEMO_PHONE").unwrap();
    let demo_password = env::var("DEMO_PASSWORD").unwrap();

    let options = SignUpWithPasswordOptions {
        email_redirect_to: Some(String::from("a_random_url")),
        ..Default::default()
    };

    let session = auth_client
        .sign_up_with_phone_and_password(demo_phone.clone(), demo_password.clone(), Some(options))
        .await;

    if session.is_err() {
        eprintln!("{:?}", session.as_ref().unwrap_err())
    }

    assert!(session.is_ok());

    let new_session = auth_client
        .sign_in_with_phone_and_password(&demo_phone, &demo_password)
        .await;

    if new_session.is_err() {
        eprintln!("{:?}", new_session.as_ref().unwrap_err())
    }

    assert!(new_session.is_ok() && new_session.unwrap().user.phone == demo_phone);

    let response = auth_client.send_sms_with_otp(demo_phone).await;

    if response.is_err() {
        eprintln!("{:?}", response.as_ref().unwrap_err())
    }

    assert!(response.is_ok())
}

#[tokio::test]
async fn send_login_email_with_magic_link() {
    let auth_client = create_test_client();

    let demo_email = env::var("DEMO_EMAIL").unwrap();

    let response = auth_client
        .send_login_email_with_magic_link(demo_email)
        .await;

    if response.is_err() {
        eprintln!("{:?}", response.as_ref().unwrap_err())
    }

    // Wait to prevent running into Supabase rate limits when running cargo test
    let one_minute = time::Duration::from_secs(60);
    thread::sleep(one_minute);

    assert!(response.is_ok())
}

#[tokio::test]
async fn send_email_with_otp() {
    let auth_client = create_test_client();

    let demo_email = env::var("DEMO_EMAIL").unwrap();

    let response = auth_client.send_email_with_otp(demo_email, None).await;

    if response.is_err() {
        eprintln!("{:?}", response.as_ref().unwrap_err())
    }

    // Wait to prevent running into Supabase rate limits when running cargo test
    let one_minute = time::Duration::from_secs(60);
    thread::sleep(one_minute);

    assert!(response.is_ok())
}

#[tokio::test]
async fn sign_in_with_oauth_test() {
    let auth_client = create_test_client();

    let mut params = HashMap::new();
    params.insert("key".to_string(), "value".to_string());
    params.insert("second_key".to_string(), "second_value".to_string());
    params.insert("third_key".to_string(), "third_value".to_string());

    let options = SignInWithOAuthOptions {
        query_params: Some(params),
        redirect_to: Some("localhost".to_string()),
        scopes: Some("repo gist notifications".to_string()),
        skip_brower_redirect: Some(true),
    };

    let response = auth_client
        .sign_in_with_oauth(supabase_auth::models::Provider::Github, Some(options))
        .await;

    if response.is_err() {
        println!("SIGN IN WITH OAUTH TEST RESPONSE -- \n{:?}", response);
    }

    assert!(response.unwrap().url.to_string().len() > 1);
}

#[tokio::test]
async fn sign_in_with_oauth_no_options_test() {
    let auth_client = create_test_client();

    // Must login to get a user bearer token
    let demo_email = env::var("DEMO_EMAIL").unwrap();
    let demo_password = env::var("DEMO_PASSWORD").unwrap();

    let session = auth_client
        .login_with_email(demo_email, demo_password)
        .await;

    if session.is_err() {
        eprintln!("{:?}", session.as_ref().unwrap_err())
    }

    let response = auth_client
        .sign_in_with_oauth(supabase_auth::models::Provider::Github, None)
        .await;

    println!(
        "SIGN IN WITH OAUTH \n NO OPTIONS TEST RESPONSE -- \n{:?}",
        response
    );

    if response.is_err() {
        eprintln!("{:?}", response.as_ref().unwrap_err())
    }

    assert!(response.is_ok())
}

#[tokio::test]
async fn get_user_test() {
    let auth_client = create_test_client();

    // Must login to get a user bearer token
    let demo_email = env::var("DEMO_EMAIL").unwrap();
    let demo_password = env::var("DEMO_PASSWORD").unwrap();

    let session = auth_client
        .login_with_email(&demo_email, &demo_password)
        .await;

    if session.is_err() {
        eprintln!("{:?}", session.as_ref().unwrap_err())
    }

    let user = auth_client
        .get_user(session.unwrap().access_token)
        .await
        .unwrap();

    assert!(user.email == demo_email)
}

#[tokio::test]
async fn update_user_test() {
    let auth_client = create_test_client();

    // Must login to get a user bearer token
    let demo_email = env::var("DEMO_EMAIL").unwrap();
    let demo_password = env::var("DEMO_PASSWORD").unwrap();

    let session = auth_client
        .login_with_email(&demo_email, &demo_password)
        .await
        .unwrap();

    eprintln!("{:?}", session);

    let updated_user = UpdateUserPayload {
        email: Some(demo_email.clone()),
        password: Some("qqqqwwww".to_string()),
        data: None,
    };

    let first_response = auth_client
        .update_user(updated_user, session.access_token)
        .await;

    if first_response.is_err() {
        eprintln!("{:?}", first_response.as_ref().unwrap_err())
    }

    // Login with new password to validate the change
    let test_password = "qqqqwwww";

    let new_session = auth_client
        .login_with_email(demo_email.as_ref(), test_password)
        .await;

    if new_session.is_err() {
        eprintln!("{:?}", new_session.as_ref().unwrap_err())
    }

    // Return the user to original condition
    let original_user = UpdateUserPayload {
        email: Some(demo_email),
        password: Some("qwerqwer".to_string()),
        data: None,
    };

    let second_response = auth_client
        .update_user(original_user, new_session.unwrap().access_token)
        .await;

    assert!(second_response.is_ok())
}

#[tokio::test]
async fn exchange_token_for_session() {
    let auth_client = create_test_client();

    let demo_email = env::var("DEMO_EMAIL").unwrap();
    let demo_password = env::var("DEMO_PASSWORD").unwrap();

    let original_session = auth_client
        .login_with_email(&demo_email, &demo_password)
        .await
        .unwrap();

    assert!(original_session.user.email == demo_email);

    let new_session = auth_client
        .refresh_session(original_session.refresh_token)
        .await
        .unwrap();

    assert!(new_session.user.email == demo_email)
}

#[tokio::test]
async fn reset_password_for_email_test() {
    let auth_client = create_test_client();

    let demo_email = env::var("DEMO_EMAIL").unwrap();

    let response = auth_client.reset_password_for_email(demo_email).await;

    // Wait to prevent running into Supabase rate limits when running cargo test
    let one_minute = time::Duration::from_secs(60);
    thread::sleep(one_minute);

    assert!(response.is_ok())
}

#[tokio::test]
async fn resend_email_test() {
    let auth_client = create_test_client();

    let uuid = uuid::Uuid::now_v7();

    let demo_email = format!("signup__{}@demo.com", uuid);
    let demo_password = "ciJUAojfZZYKfCxkiUWH";

    let session = auth_client
        .sign_up_with_email_and_password(demo_email.clone(), demo_password.to_string(), None)
        .await;

    if session.is_err() {
        eprintln!("{:?}", session.as_ref().unwrap_err())
    }

    let credentials = ResendParams {
        otp_type: supabase_auth::models::OtpType::Signup,
        email: demo_email.to_owned(),
        options: None,
    };

    // Wait to prevent running into Supabase rate limits when running cargo test
    let one_minute = time::Duration::from_secs(60);
    thread::sleep(one_minute);

    let response = auth_client.resend(credentials).await;

    if response.is_err() {
        println!("{:?}", response)
    }

    assert!(response.is_ok() && session.unwrap().user.email == demo_email)
}

#[tokio::test]
async fn logout_test() {
    let auth_client = create_test_client();

    let demo_email = env::var("DEMO_EMAIL").unwrap();
    let demo_password = env::var("DEMO_PASSWORD").unwrap();

    let session = auth_client
        .login_with_email(demo_email, demo_password.to_string())
        .await
        .unwrap();

    let logout = auth_client
        .logout(Some(LogoutScope::Global), session.access_token)
        .await;

    if logout.is_err() {
        println!("{:?}", logout)
    }

    assert!(logout.is_ok())
}

#[tokio::test]
async fn test_sso_login() {
    let auth_client = create_test_client();
    let demo_domain = env::var("DEMO_DOMAIN").unwrap();
    let params = SignInWithSSO {
        domain: Some(demo_domain),
        options: None,
        provider_id: None,
    };

    let url = auth_client.sso(params).await.unwrap();

    println!("{}", url.to_string());

    assert!(url.to_string().len() > 1);
}

#[tokio::test]
async fn invite_by_email_test() {
    let auth_client = create_test_client();

    let demo_email = env::var("DEMO_INVITE").unwrap();

    println!("{}", auth_client.api_key());

    let user = auth_client
        // NOTE: Requires admin permissions to issue invites
        .invite_user_by_email(&demo_email, None, auth_client.api_key())
        .await
        .unwrap();

    assert!(user.email == demo_email)
}

#[tokio::test]
async fn get_settings_test() {
    let auth_client = create_test_client();

    let settings = auth_client.get_settings().await.unwrap();

    assert!(settings.external.github == true)
}

#[tokio::test]
async fn get_health_test() {
    let auth_client = create_test_client();

    let health = auth_client.get_health().await.unwrap();

    assert!(health.description != "")
}
