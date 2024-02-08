//! Database Operation Testing
//!
//! Could be done inside db.rs, but this feels cleaner 
//!
//! Test Naming
//! test_<function>_<case>

use diesel::Connection;
use salamandra_server::models::user::User;
use salamandra_server::db::{establish_connection, insert_new_user, select_user};

const TEST_UUID: uuid::Uuid = uuid::Uuid::from_bytes([
    0x12, 0x3e, 0x45, 0x67,             // first group: 123e4567
    0xe8, 0x9b,                         // second group: e89b
    0x12, 0xd3,                         // third group: 12d3
    0xa4, 0x56,                         // fourth group: a456
    0x42, 0x66, 0x14, 0x17, 0x40, 0x00, // fifth group: 426614174000
]);


#[test]
fn test_insert_new_user_sucess() {
    let conn = &mut establish_connection().unwrap();

    conn.test_transaction::<_, diesel::result::Error, _>(|conn| {
        // Create a new user object to insert
        let new_user = User {
            id: TEST_UUID,
            username: "Test username".to_string(),
            display_name: "Test username".to_string(),        
            date_joined: chrono::Utc::now(),
            training_state: 0,
            fitness_level: 0,
            pfp_url: None,
            date_of_birth: None,
            height: None,
        };   

        let insert_res = insert_new_user(conn, new_user);
        assert!(insert_res.is_ok());

        let read_res = select_user(conn, TEST_UUID);
        assert!(read_res.is_ok());

        assert_eq!(read_res.unwrap().len(), 1, "Expected one user to be found");
        Ok(())
    });
}

#[test]
fn test_select_user_existing_user() {
    let conn = &mut establish_connection().unwrap();

    conn.test_transaction::<_, diesel::result::Error, _>(|conn| {
        // Create a new user object to insert
        let new_user = User {
            id: TEST_UUID,
            username: "Test username".to_string(),
            display_name: "Test username".to_string(),        
            date_joined: chrono::Utc::now(),
            training_state: 0,
            fitness_level: 0,
            pfp_url: None,
            date_of_birth: None,
            height: None,
        };   

        let insert_res = insert_new_user(conn, new_user);
        assert!(insert_res.is_ok());

        let read_res = select_user(conn, TEST_UUID);
        assert!(read_res.is_ok());
        let vec = read_res.unwrap();
        assert_eq!(vec.len(), 1, "Should only be one user with this UUID");
        let read_user: User = vec.first().unwrap().clone();
        assert_eq!(read_user.id, TEST_UUID, "Didn't select expected user");

        Ok(())
    });
}


#[test]
fn test_select_user_non_existing_user() {
    let conn = &mut establish_connection().unwrap();

    conn.test_transaction::<_, diesel::result::Error, _>(|conn| {
        // Look up non existing user
        let read_res = select_user(conn, TEST_UUID);
        assert!(read_res.is_ok());
        let vec = read_res.unwrap();
        assert_eq!(vec.len(), 0, "Should only be one user with this UUID");

        Ok(())
    });
}
