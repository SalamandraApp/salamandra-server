-- Your SQL goes here
CREATE TABLE Users (
    id UUID PRIMARY KEY,
    username VARCHAR(255) UNIQUE NOT NULL,
    display_name VARCHAR(255),
    pfp_url TEXT,
    date_of_birth DATE,
    date_joined TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    training_state INT,
    fitness_level INT,
    height INT
);
