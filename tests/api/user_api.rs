use crate::helpers::TestApp;

#[tokio::test]
async fn add_user_returns_201_for_valid_data() {
    // Arrange
    let app = TestApp::spawn().await;
    let client = reqwest::Client::new();

    // Act
    let body = r#"{
        "email_address":"user@example.com",
        "username":"user"
    }"#;
    let response = client
        .post(app.url("/api/users"))
        .header("Content-Type", "application/json")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    let response_status = response.status().as_u16();
    let response_text = response.text().await.unwrap();
    println!("{}", response_text);
    assert_eq!(201, response_status);
    let saved = sqlx::query!("SELECT email, username FROM users;")
        .fetch_one(&app.db_pool)
        .await
        .unwrap();
    assert_eq!(saved.email, "user@example.com");
    assert_eq!(saved.username, "user");
}
