-- Your SQL goes here
CREATE EXTENSION IF NOT EXISTS pgcrypto;
CREATE TABLE Users (
    id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
    username VARCHAR(255) UNIQUE NOT NULL,
    display_name VARCHAR(255) NOT NULL,
    date_joined TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    date_of_birth DATE,
    height INT,
    weight FLOAT4,
    gender INT,
    fitness_goal INT NOT NULL,
    fitness_level INT NOT NULL
);
