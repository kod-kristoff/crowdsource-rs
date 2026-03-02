use crate::helpers::spawn_app;

#[tokio::test]
async fn add_user_returns_201_for_valid_data() {
    // Arrange
    let app = spawn_app().await;
    let body = r#"{
        "email_address":"user@example.com",
        "username":"user"
    }"#;

    // Act
    let response = app.post_users(body.into()).await;

    // Assert
    let response_status = response.status().as_u16();
    let response_text = response.text().await.unwrap();
    println!("{}", response_text);
    assert_eq!(response_status, 201);
    let saved = sqlx::query!("SELECT email, username FROM users;")
        .fetch_one(&app.db_pool)
        .await
        .unwrap();
    assert_eq!(saved.email, "user@example.com");
    assert_eq!(saved.username, "user");
}

#[tokio::test]
async fn add_user_returns_422_for_invalid_data() {
    // Arrange
    let app = spawn_app().await;
    let test_cases = vec![
        (
            r#"{"email_address":"user@example.com"}"#,
            "missing username",
        ),
        (r#"{"username":"user"}"#, "missing email"),
        (
            r#"{
                "email_address":"user-example.com",
                "username":"user"
            }"#,
            "invalid email",
        ),
        (
            r#"{
                "email_address":"user@example.com",
                "username":""
            }"#,
            "empty username",
        ),
        (
            r#"{
                "email_address":"user@example.com",
                "username":"with whitespace"
            }"#,
            "username with whitespace",
        ),
    ];
    for (invalid_body, error_message) in test_cases {
        // Act
        let response = app.post_users(invalid_body.into()).await;

        // Assert
        let actual_status = response.status().as_u16();
        let actual_msg: serde_json::Value = response.json().await.unwrap();
        assert_eq!(
            actual_status, 422,
            "The API did not fail with 422 Unprocessable Entity when the payload was missing {}",
            error_message
        );
        insta::assert_json_snapshot!(
            format!("add_user_returns_422_for_invalid_data-{}", error_message),
            actual_msg
        );
    }
}

#[tokio::test]
async fn add_user_fails_with_500_if_there_is_a_fatal_database_error() {
    // Arrange
    let app = spawn_app().await;
    let body = r#"{
        "email_address":"user@example.com",
        "username":"user"
    }"#;
    // Sabotage the database
    sqlx::query!("ALTER TABLE users DROP COLUMN email;")
        .execute(&app.db_pool)
        .await
        .unwrap();

    // Act
    let response = app.post_users(body.into()).await;

    // Assert
    assert_eq!(response.status().as_u16(), 500);
    let actual_msg: serde_json::Value = response.json().await.unwrap();

    insta::assert_json_snapshot!(actual_msg);
}

#[tokio::test]
async fn add_user_returns_422_for_existing_data() {
    // Arrange
    let app = spawn_app().await;
    let body = r#"{
        "email_address":"user@example.com",
        "username":"user"
    }"#;
    app.post_users(body.into()).await;
    let test_cases = vec![
        (
            r#"{
         "email_address":"user@example.com",
         "username":"user1"
     }"#,
            "duplicate email",
        ),
        (
            r#"{
         "email_address":"user1@example.com",
         "username":"user"
     }"#,
            "duplicate username",
        ),
    ];

    for (duplicate_body, error_message) in test_cases {
        // Act
        let response = app.post_users(duplicate_body.into()).await;

        // Assert
        let response_status = response.status().as_u16();
        let actual_msg: serde_json::Value = response.json().await.unwrap();
        assert_eq!(
            response_status, 422,
            "The API didn't fail with 422 Unprocessable Entity when the payload was {}.",
            error_message
        );
        insta::assert_json_snapshot!(
            format!("add_user_returns_422_for_existing_data-{}", error_message),
            actual_msg
        );
    }
}
