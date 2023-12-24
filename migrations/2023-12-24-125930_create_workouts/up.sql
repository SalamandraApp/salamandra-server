-- Your SQL goes here
CREATE TABLE workouts (
    id SERIAL PRIMARY KEY,
    user_id INT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users (id)
);

