//! Database Operation Testing
//!
//! Could be done inside db.rs, but this feels cleaner 
//!
//! Test Naming
//! test_<function>_<case>

use testcontainers::clients::Cli;

use salamandra_server::models::user::User;
use salamandra_server::db::{establish_connection, insert_new_user, select_user};
use salamandra_server::utils::test as common;

const TEST_UUID: uuid::Uuid = uuid::Uuid::from_bytes([
    0x12, 0x3e, 0x45, 0x67,             
    0xe8, 0x9b,                         
    0x12, 0xd3,                         
    0xa4, 0x56,                         
    0x42, 0x66, 0x14, 0x17, 0x40, 0x00, 
]);


#[test]
fn test_insert_new_user_sucess() {
    let docker = Cli::default();
    let image = common::container_setup();
    let instance = docker.run(image.image);
    let url = format!(
        "postgres://{}:password@127.0.0.1:{}/{}",
        image.user,
        instance.get_host_port_ipv4(5432),
        image.db
    );
    let mut conn = establish_connection(url).unwrap();
    common::run_migrations(&mut conn)
        .expect("Failed to run migrations");
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

    let insert_res = insert_new_user(&mut conn, new_user);
    assert!(insert_res.is_ok());

    let read_res = select_user(&mut conn, TEST_UUID);
    assert!(read_res.is_ok());

    assert_eq!(read_res.unwrap().len(), 1, "Expected one user to be found");
}

#[test]
fn test_select_user_existing_user() {
    let docker = Cli::default();
    let image = common::container_setup();
    let instance = docker.run(image.image);
    let url = format!(
        "postgres://{}:password@127.0.0.1:{}/{}",
        image.user,
        instance.get_host_port_ipv4(5432),
        image.db
    );
    let mut conn = establish_connection(url).unwrap();
    common::run_migrations(&mut conn)
        .expect("Failed to run migrations");
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

    let insert_res = insert_new_user(&mut conn, new_user);
    assert!(insert_res.is_ok());

    let read_res = select_user(&mut conn, TEST_UUID);
    assert!(read_res.is_ok());
    let vec = read_res.unwrap();
    assert_eq!(vec.len(), 1, "Should only be one user with this UUID");
    let read_user: User = vec.first().unwrap().clone();
    assert_eq!(read_user.id, TEST_UUID, "Didn't select expected user");
}


#[test]
fn test_select_user_non_existing_user() {    
    let docker = Cli::default();
    let image = common::container_setup();
    let instance = docker.run(image.image);
    let url = format!(
        "postgres://{}:password@127.0.0.1:{}/{}",
        image.user,
        instance.get_host_port_ipv4(5432),
        image.db
    );
    let mut conn = establish_connection(url).unwrap();
    common::run_migrations(&mut conn)
        .expect("Failed to run migrations");
    // Look up non existing user
    let read_res = select_user(&mut conn, TEST_UUID);
    assert!(read_res.is_ok());
    let vec = read_res.unwrap();
    assert_eq!(vec.len(), 0, "Should only be one user with this UUID");

}
