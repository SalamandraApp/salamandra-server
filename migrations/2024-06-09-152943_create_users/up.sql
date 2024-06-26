-- Your SQL goes here
CREATE EXTENSION IF NOT EXISTS pgcrypto;
CREATE TABLE Users (
    id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
    username VARCHAR UNIQUE NOT NULL,
    display_name VARCHAR NOT NULL,
    date_joined DATE DEFAULT CURRENT_DATE NOT NULL,
    date_of_birth DATE,
    height INT,
    weight FLOAT4,
    gender INT,
    fitness_goal INT,
    fitness_level INT
);
