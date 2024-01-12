-- Your SQL goes here
CREATE TABLE Users (
    id UUID PRIMARY KEY,
    username VARCHAR(255) UNIQUE NOT NULL,
    display_name VARCHAR(255) NOT NULL,
    pfp_url TEXT,
    date_of_birth DATE,
    date_joined TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    training_state INT NOT NULL,
    fitness_level INT NOT NULL,
    height INT
);
