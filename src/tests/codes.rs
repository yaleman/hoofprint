use super::{TEST_USER_EMAIL, TEST_USER_PASSWORD};

use sea_orm::EntityTrait;
use uuid::Uuid;

use crate::{
    Code,
    db::entities::code,
    prelude::Urls,
    tests::{setup_test_server, setup_test_user},
    web::{auth::LoginForm, forms::CreateCodeForm},
};

#[tokio::test]
async fn test_code_create() {
    let (server, db) = setup_test_server().await;
    let _user = setup_test_user(db.clone()).await;

    let response = server
        .post(Urls::Login.as_ref())
        .form(&LoginForm {
            email: TEST_USER_EMAIL.to_string(),
            password: TEST_USER_PASSWORD.to_string(),
            error: None,
            success: None,
        })
        .await;
    dbg!(&response);
    assert_eq!(response.status_code(), 303);

    // create a new code
    let response = server
        .post(Urls::Create.as_ref())
        .form(&CreateCodeForm {
            code_type: Code::Bar.to_string(),
            code_value: "123456".to_string(),
            site_id: "00000000-0000-0000-0000-000000000000".to_string(),
            code_name: Some("Test Code".to_string()),
        })
        .await;
    dbg!(&response);
    assert_eq!(response.status_code(), 303);
    let location = response
        .headers()
        .get("Location")
        .expect("Location header missing")
        .to_str()
        .expect("Failed to convert Location header to str");
    assert!(location.starts_with("/view/"));
    let code = location.trim_start_matches("/view/");
    let code_uuid = uuid::Uuid::parse_str(code).expect("Failed to parse code UUID");
    let code_in_db = code::Entity::find_by_id(code_uuid)
        .one(&db)
        .await
        .expect("Failed to query code")
        .expect("Code not found in database");
    assert_eq!(code_in_db.id, code_uuid);
    // check we can get the code view page
    let response = server.get(location).await;
    assert_eq!(response.status_code(), 200);

    let response = server.get(Urls::Home.as_ref()).await;
    assert_eq!(response.status_code(), 200);
    response.assert_text_contains("Test Code");
    response.assert_text_contains("123456");
    response.assert_text_contains(code);

    // try to view a code that doesn't belong to us
    let admin_code = code::Model::create_new(
        db.clone(),
        Uuid::nil(),
        Code::Bar,
        "hello-admin",
        uuid::Uuid::nil(),
        Some("Admin Code"),
    )
    .await
    .expect("Failed to create admin code");

    let response = server.get(&format!("/view/{}", admin_code.id)).await;
    assert_eq!(response.status_code(), 404); // because it's not our code!
}
